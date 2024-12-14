# ts3_users_tracker

## Installing

```sh
git clone https://github.com/tsorak/ts3_users_tracker
cd pkg
makepkg -sic
```

**IMPORTANT!**

As a teamspeak server moderator you have to enable Client logging manually.

1. Connect to the server and right-click the servername (topmost entry in the channel list).
2. Click "Edit Virtual Server".
3. Go to the "Logs" tab.
4. Checkmark the "Clients" option.
5. Hit "Apply" and you're done.

## Usage

### The systemd service

The `ts3_users_tracker.service` gets installed to `/usr/lib/systemd/system/`. It will have to be manually enabled and started.

By default it starts a webserver on port 80, tracking the default teamspeak server unit. Changing the port can be done with `-p`.

**WARNING:** There is no authentication on the webserver meaning anyone with access can view who is online.

To disable the webserver remove the `--serve-http` flag.

### The binary

Available options can be printed with `ts3_users_tracker -h`.

The binary can be run as is without arguments to continuously print a table of which users are currently online.

Example output:

```
------Online users------
    11:03:45 14 Dec    
------------------------
[11:03:45 14 Dec] tsorak
```

If your Teamspeak service is something else than "teamspeak3-server.service" it can be specified with `-u`.

The `--since` flag specifies a date to start reading logs from. The start date of the server unit is used by default.

If your server has been running for a couple months you can reduce the time reading logs by passing `--since today`, `--since -3d`, `--since -6h` etc...
See the journalctl `--since` flag for valid timestamp formatting.

## What it can not do

Show online users on specific virtual-servers. The project is built and tested for tracking users in a single virtual-server. When running with multiple virtual-servers I would assume users across all of them are tracked.

