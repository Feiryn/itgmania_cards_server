#include "global.h"
#include "MemoryCardDriverThreaded_Linux.h"
#include "RageLog.h"
#include "RageUtil.h"
#include "RageTimer.h"
#include "GameState.h"

#include <cerrno>
#include <climits>
#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/socket.h>
#include <sys/un.h>
#if defined(HAVE_FCNTL_H)
#include <fcntl.h>
#endif
#if defined(HAVE_DIRENT_H)
#include <dirent.h>
#endif

static RString LastP1CardNumber = "0";
static RString LastP2CardNumber = "0";
static int LastSentState = -1;
static int g_SocketFd = -1;

static RString ExpandHomePath(const RString& path)
{
	if (path.empty() || path[0] != '~')
		return path;
	
	const char* home = getenv("HOME");
	if (!home)
	{
		LOG->Warn("ExpandHomePath: HOME environment variable not set");
		return path;
	}
	
	// Replace ~ with the home directory
	if (path.length() == 1 || path[1] == '/')
		return RString(home) + path.substr(1);
	
	return path;
}

static bool ConnectToSocketIfNeeded()
{
	if (g_SocketFd != -1)
		return true;

	LOG->Trace("ConnectToSocketIfNeeded: Creating new socket connection");
	
	// Create socket
	g_SocketFd = socket(AF_UNIX, SOCK_STREAM, 0);
	if (g_SocketFd == -1)
	{
		LOG->Warn("ConnectToSocketIfNeeded: Unable to create socket: %s", strerror(errno));
		return false;
	}
	
	// Setup socket address
	struct sockaddr_un addr;
	memset(&addr, 0, sizeof(addr));
	addr.sun_family = AF_UNIX;
	strncpy(addr.sun_path, "/tmp/itgmania_cards.sock", sizeof(addr.sun_path) - 1);
	
	// Connect to socket
	if (connect(g_SocketFd, (struct sockaddr*)&addr, sizeof(addr)) == -1)
	{
		LOG->Warn("ConnectToSocketIfNeeded: Unable to connect to socket: %s", strerror(errno));
		close(g_SocketFd);
		g_SocketFd = -1;
		return false;
	}
	
	LOG->Trace("ConnectToSocketIfNeeded: Successfully connected to socket");
	return true;
}

static void SendSocketCommand(const RString& command)
{
	LOG->Trace("SendSocketCommand: Sending command: %s", command.c_str());
	
	if (!ConnectToSocketIfNeeded())
	{
		LOG->Warn("SendSocketCommand: Failed to connect to socket");
		return;
	}
	
	int sent = write(g_SocketFd, command.c_str(), command.length());
	if (sent == -1)
	{
		LOG->Warn("SendSocketCommand: Error writing to socket: %s", strerror(errno));
		close(g_SocketFd);
		g_SocketFd = -1;
	}
	else
	{
		LOG->Trace("SendSocketCommand: Successfully sent %d bytes", sent);
	}

	char buf[1024];
	int iGot = read(g_SocketFd, buf, sizeof(buf));
	if (iGot == -1)
	{
		LOG->Warn("SendSocketCommand: Error reading from socket: %s", strerror(errno));
		close(g_SocketFd);
		g_SocketFd = -1;
	}
	else if (iGot == 0)
	{
		LOG->Warn("SendSocketCommand: Socket closed by server");
		close(g_SocketFd);
		g_SocketFd = -1;
	}
	else if (buf[0] != 'O' || buf[1] != 'K')
	{
		LOG->Warn("SendSocketCommand: Received error response from server: %.*s", iGot, buf);
	}
}

static std::vector<RString> ReadUnixSocketCards()
{
	LOG->Trace("ReadUnixSocketCards: Reading card data from Unix socket");
	RString sBuf;
	
	if (!ConnectToSocketIfNeeded())
	{
		LOG->Warn("ReadUnixSocketCards: Unable to connect to socket");
		return std::vector<RString>{ "0", "0" };
	}
	
	// Send READ command
	const char* readCmd = "READ\n";
	int sent = write(g_SocketFd, readCmd, strlen(readCmd));
	if (sent == -1)
	{
		LOG->Warn("ReadUnixSocketCards: Error writing READ command: %s", strerror(errno));
		close(g_SocketFd);
		g_SocketFd = -1;
		return std::vector<RString>{ "0", "0" };
	}

	while(1)
	{
		char buf[1024];
		int iGot = read(g_SocketFd, buf, sizeof(buf));
		if (iGot == -1)
		{
			LOG->Warn("ReadUnixSocketCards: Error reading socket: %s", strerror(errno));
			close(g_SocketFd);
			g_SocketFd = -1;
			return std::vector<RString>{ "0", "0" };
		}
		if (iGot == 0)
		{
			LOG->Warn("ReadUnixSocketCards: Socket closed by server");
			close(g_SocketFd);
			g_SocketFd = -1;
			return std::vector<RString>{ "0", "0" };
		}

		sBuf.append(buf, iGot);
		if (iGot < (int) sizeof(buf))
			break;
	}

	std::vector<RString> asLines;
	split( sBuf, ",", asLines );
	if( asLines.size() < 2 )
	{
		LOG->Warn("ReadUnixSocketCards: Invalid data format, returning null devices");
		return std::vector<RString>{ "0", "0" };
	}
	Trim(asLines[0]);
	Trim(asLines[1]);

	LOG->Trace("ReadUnixSocketCards: Successfully read cards: P1=%s, P2=%s", asLines[0].c_str(), asLines[1].c_str());
	return std::vector<RString>{ asLines[0], asLines[1] };
}

static bool ReadFile( const RString &sPath, RString &sBuf )
{
	LOG->Trace("ReadFile: Reading file: %s", sPath.c_str());
	sBuf.clear();

	int fd = open( sPath.c_str(), O_RDONLY );
	if( fd == -1 )
	{
		// "No such file or directory" is understandable
		if (errno != ENOENT)
			LOG->Warn( "Error opening \"%s\": %s", sPath.c_str(), strerror(errno) );
		return false;
	}

	while(1)
	{
		char buf[1024];
		int iGot = read( fd, buf, sizeof(buf) );
		if( iGot == -1 )
		{
			close(fd);
			LOG->Warn( "Error reading \"%s\": %s", sPath.c_str(), strerror(errno) );
			return false;
		}

		sBuf.append( buf, iGot );
		if( iGot < (int) sizeof(buf) )
			break;
	}

	close(fd);
	LOG->Trace("ReadFile: Successfully read %d bytes from %s", (int)sBuf.size(), sPath.c_str());
	return true;
}

static void GetFileList( const RString &sPath, std::vector<RString> &out )
{
	LOG->Trace("GetFileList: Getting file list from: %s", sPath.c_str());
	out.clear();

	DIR *dp = opendir( sPath.c_str() );
	if( dp == nullptr )
	{
		LOG->Trace("GetFileList: Unable to open directory: %s", sPath.c_str());
		return; // false; // XXX warn
	}

	while( const struct dirent *ent = readdir(dp) )
		out.push_back( ent->d_name );

	closedir( dp );
	LOG->Trace("GetFileList: Found %d files in %s", (int)out.size(), sPath.c_str());
}

bool MemoryCardDriverThreaded_Linux::TestWrite( UsbStorageDevice* pDevice )
{
	return true;
}

bool MemoryCardDriverThreaded_Linux::USBStorageDevicesChanged()
{
	bool player1Enabled = GAMESTATE->IsPlayerEnabled(PLAYER_1);
	if (!player1Enabled && LastP1CardNumber != "0")
	{
		LOG->Trace("USBStorageDevicesChanged: Player 1 unjoined, sending card reset");
		SendSocketCommand("RESET 1\n");
	}

	bool player2Enabled = GAMESTATE->IsPlayerEnabled(PLAYER_2);
	if (!player2Enabled && LastP2CardNumber != "0")
	{
		LOG->Trace("USBStorageDevicesChanged: Player 2 unjoined, sending card reset");
		SendSocketCommand("RESET 2\n");
	}

	if ((LastSentState == -1 || LastSentState == 1) && !player1Enabled && !player2Enabled) {
		SendSocketCommand("DISABLE\n");
		LastSentState = 0;
	} else if (LastSentState == 0 && (player1Enabled || player2Enabled)) {
		SendSocketCommand("ENABLE\n");
		LastSentState = 1;
	}

	LOG->Trace("USBStorageDevicesChanged: Checking for USB storage device changes");
	std::vector<RString> currentCards = ReadUnixSocketCards();

	if (currentCards.size() != 2)
	{
		LOG->Trace("USBStorageDevicesChanged: Invalid card count (%d), devices changed", (int)currentCards.size());
		return true;
	}

	if (currentCards[0].compare(LastP1CardNumber) != 0 || currentCards[1].compare(LastP2CardNumber) != 0)
	{
		LOG->Trace("USBStorageDevicesChanged: Card change detected - P1: %s->%s, P2: %s->%s", 
			LastP1CardNumber.c_str(), currentCards[0].c_str(),
			LastP2CardNumber.c_str(), currentCards[1].c_str());
		LastP1CardNumber = currentCards[0];
		LastP2CardNumber = currentCards[1];
		return true;
	}

	LOG->Trace("USBStorageDevicesChanged: No changes detected");
	return false;
}

void MemoryCardDriverThreaded_Linux::GetUSBStorageDevices( std::vector<UsbStorageDevice>& vDevicesOut )
{
	LOG->Trace("GetUSBStorageDevices: Getting USB storage devices");
	vDevicesOut.clear();

	if (LastP1CardNumber != "0")
	{
		LOG->Trace("GetUSBStorageDevices: Adding P1 device with card number: %s", LastP1CardNumber.c_str());
		UsbStorageDevice player1Device;
		player1Device.sDevice = LastP1CardNumber;
		RString mountPath = ExpandHomePath("~/.itgmania_cards/accounts/" + LastP1CardNumber);
		player1Device.SetOsMountDir(mountPath);
		player1Device.iBus = 1;
		vDevicesOut.push_back(player1Device);
	}

	if (LastP2CardNumber != "0")
	{
		LOG->Trace("GetUSBStorageDevices: Adding P2 device with card number: %s", LastP2CardNumber.c_str());
		UsbStorageDevice player2Device;
		player2Device.sDevice = LastP2CardNumber;
		RString mountPath = ExpandHomePath("~/.itgmania_cards/accounts/" + LastP2CardNumber);
		player2Device.SetOsMountDir(mountPath);
		player2Device.iBus = 2;
		vDevicesOut.push_back(player2Device);
	}

	LOG->Trace("GetUSBStorageDevices: Returning %d devices", (int)vDevicesOut.size());
}


bool MemoryCardDriverThreaded_Linux::Mount( UsbStorageDevice* pDevice )
{
	LOG->Trace("Mount: Mounting device : %s", pDevice ? pDevice->sDevice.c_str() : "null");
	return true;
}

void MemoryCardDriverThreaded_Linux::Unmount( UsbStorageDevice* pDevice )
{
	LOG->Trace("Is device name available? %s", pDevice ? (pDevice->bIsNameAvailable ? "yes" : "no") : "null device");

	LOG->Trace("Unmount: Unmounting device : %s", pDevice ? pDevice->sDevice.c_str() : "null");
}
