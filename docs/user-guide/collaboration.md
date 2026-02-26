# Real-Time Collaboration

TaleNode supports real-time collaboration over your local network. One user hosts a session, and others join to edit the same graph simultaneously. Changes sync instantly via WebSocket.

## Concepts

### Host / Client Model

- **Host**: One user starts a server on their machine. Their current graph becomes the shared project
- **Client**: Other users connect to the host's IP address and port. They receive a full copy of the graph on connect

The host is the authoritative source of truth. All operations are broadcast through the host's server.

### Conflict Resolution

TaleNode uses **Last-Write-Wins (LWW)** by timestamp. If two users edit the same node field simultaneously, the most recent edit takes effect. In practice, peer selection awareness (see below) helps avoid conflicts.

## Setup

### Collaboration Settings

Open **Settings > Settings...** and configure:

| Field | Description |
|---|---|
| **Username** | Your display name visible to other collaborators |
| **Default Port** | Port number for hosting (default: `9847`) |

### Network Requirements

- All collaborators must be on the same local network (LAN)
- The host's firewall must allow incoming TCP connections on the chosen port
- No internet connection required — collaboration works entirely over LAN

## Hosting a Session

### From the Menu

1. Go to **Collaborate > Host Session**
2. TaleNode starts a WebSocket server on your configured port
3. Share your local IP address and port with collaborators

### From the Collaboration Panel

1. Open via **View > Collaboration**
2. Set the port number
3. Click **Start Hosting**

The status bar shows "Hosting on 0.0.0.0:{port}" when the server is running.

## Joining a Session

### From the Menu

1. Go to **Collaborate > Join Session**
2. Enter the host's IP address and port

### From the Collaboration Panel

1. Open via **View > Collaboration**
2. Enter the host's IP address and port
3. Click **Join**

On connect, you receive a full copy of the host's graph. Your local graph is replaced with the shared version.

## Collaboration Panel

Open via **View > Collaboration** (or drag the tab from the dock).

### Offline View

When not connected, the panel shows:

- **Host** section: port input + **Start Hosting** button
- **Join** section: host address input, port input + **Join** button

### Connected View

When hosting or joined, the panel shows:

- **Status**: "Hosting on ..." or "Joined to ..."
- **Peer List**: All connected users with colored indicators
- **Disconnect** button

Each peer shows:

| Info | Description |
|---|---|
| **Color dot** | Unique color assigned to the peer |
| **Username** | The peer's display name |

## What Syncs

All graph operations sync in real time:

| Operation | Synced? |
|---|---|
| Add/remove/move nodes | Yes |
| Edit node fields (text, speaker, etc.) | Yes |
| Add/remove connections | Yes |
| Add/remove/edit variables | Yes |
| Add/remove/edit characters | Yes |
| Cursor/selection updates | Yes |
| Undo/redo | Local only — not broadcast |
| Canvas pan/zoom | Local only |

## Peer Selection Awareness

When a peer selects nodes, their selection is broadcast to all other users. You can see which nodes other collaborators are working on through the peer list's `selected_nodes` field. This helps avoid editing conflicts.

## Disconnecting

- **Menu**: Go to **Collaborate > Disconnect**
- **Panel**: Click **Disconnect** in the Collaboration panel
- **Host quits**: All clients are disconnected automatically

After disconnecting, you keep a local copy of the graph in its last-synced state.

## Technical Details

### Protocol

Communication uses JSON messages over WebSocket (TCP). Message types:

| Message | Purpose |
|---|---|
| `FullSync` | Sent to new clients on connect — contains the entire graph state and peer list |
| `Operation` | A single graph operation (add/remove/move/edit) with sender name and timestamp |
| `Ack` | Acknowledgement of a received operation |
| `PeerJoined` | Notification that a new peer has connected |
| `PeerLeft` | Notification that a peer has disconnected |
| `CursorUpdate` | Selection update from a peer |
| `RequestSync` | Client requests a fresh full sync from the host |

### Operations

Each graph change is encoded as a granular operation:

`AddNode`, `RemoveNode`, `MoveNode`, `AddConnection`, `RemoveConnection`, `EditNodeField`, `AddVariable`, `RemoveVariable`, `EditVariable`, `AddCharacter`, `RemoveCharacter`, `EditCharacter`

## Tips

!!! tip
    Set distinct usernames in Settings before collaborating — this makes it easy to identify who is editing what in the peer list.

!!! tip
    Remote operations are not undoable locally. Only your own changes can be undone with Edit > Undo. Coordinate with your team if a change needs to be reverted.

!!! tip
    For the best experience, save your project before starting a collaboration session. If you're joining, your local graph will be replaced with the host's version.
