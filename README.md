# wakeonlan-relay

A small UDP relay that listens for [Wake-on-LAN](https://en.wikipedia.org/wiki/Wake_on_LAN) magic packets on one network interface and re-broadcasts them onto another.

Useful when a WoL sender (phone, a remote server, an automation tool) can't reach the target host's broadcast address directly — usually because it lives on a different subnet or VLAN.

## What it does

```text
┌──────────────┐   UDP/9    ┌───────────────────┐   UDP/7   ┌─────────────┐
│ WoL sender   │ ---------> │ wakeonlan-relay   │ --------> │ Target host │
│ (any subnet) │            │ (this program)    │ broadcast │ (LAN)       │
└──────────────┘            └───────────────────┘           └─────────────┘
```

1. Bind a UDP socket on `--listen`.
2. Receive packets from anyone who can reach that socket.
3. Forward accepted packets as UDP broadcasts to `--broadcast`.

The relay forwards the full payload it received, unchanged.

## Usage

```bash
wakeonlan-relay --listen 0.0.0.0:9 --broadcast 192.168.1.255:7
```

> **Note:** `--listen` and `--broadcast` must use **different** ports. Using the same port causes the relayed packet to be delivered right back to the listener socket, producing an infinite loop of duplicate forwards.

## Options

| Flag               | Description                              |
| ------------------ | ---------------------------------------- |
| `--listen`         | UDP socket to receive WoL packets on     |
| `--broadcast`      | Broadcast socket to relay WoL packets to |
| `--detach` / `-d`  | Run detached in the background           |
| `--help` / `-h`    | Print help                               |
| `--version` / `-V` | Print version                            |

## Logging

The program uses the [`tracing`](https://docs.rs/tracing) crate with a default-formatted subscriber. The default level is `info`. Override with the standard `RUST_LOG` environment variable:

```bash
RUST_LOG=debug wakeonlan-relay --listen 0.0.0.0:9 --broadcast 192.168.1.255:7
```

Log levels: `info`, `debug`, `warn`, `error`.

Set `RUST_LOG=wakeonlan_relay=trace` to enable every event for this binary.

## License

MIT
