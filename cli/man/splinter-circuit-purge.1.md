% SPLINTER-CIRCUIT-PURGE(1) Cargill, Incorporated | Splinter Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**splinter-circuit-purge** — Submits a request to purge the specified circuit.

SYNOPSIS
========
**splinter circuit purge** \[**FLAGS**\] \[**OPTIONS**\] CIRCUIT-ID

DESCRIPTION
===========
Request to purge a circuit by specifying the circuit ID of the circuit to be
removed from the node's storage. Internal service data associated with the
circuit is also purged. A circuit is only available to be purged if it has
already been deactivated and is only available locally. Disbanding a circuit
removes a circuit's networking functionality, allowing for a circuit to be
purged. A circuit may also be abandoned, causing the circuit's networking
capability to be disabled for the abandoning node, enabling the abandoning node
to purge the deactivated circuit.

The generated ID of an existing deactivated circuit can be viewed using the
`splinter-circuit-list`, with the `--circuit-status` option of either
`disbanded` and/or `abandoned`.

The purge request is only available for members of the node, as the circuit is
only available to the node locally. If the circuit is still considered active,
it is not able to be purged. Once a circuit has been purged, the circuit is
removed from the node's admin store and any internal Splinter service data will
also be removed. If a circuit is using the Scabbard service, for example, the
state LMDB files associated with the circuit are deleted. After purging, the
circuit and internal service data are no longer available.

FLAGS
=====
`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Decrease verbosity (the opposite of -v). When specified, only errors or
  warnings will be output.

`-V`, `--version`
: Prints version information.

`-v`
: Increases verbosity (the opposite of -q). Specify multiple times for more
  output.

OPTIONS
=======
`-k`, `--key` PRIVATE-KEY-FILE
: Specifies the full path to the private key file.

`-U`, `--url` URL
: Specifies the URL for the `splinterd` REST API. The URL is required unless
  `$SPLINTER_REST_API_URL` is set.

ARGUMENTS
=========
`CIRCUIT-ID`
: Specify the circuit ID of the circuit to be purged.

EXAMPLES
========
* The existing inactive circuit has ID `1234-ABCDE`.

The following command displays a member node requesting to purge the circuit:
```
$ splinter circuit purge \
  --key MEMBER-NODE-PRIVATE-KEY-FILE \
  --url URL-of-member-node-splinterd-REST-API \
  1234-ABCDE \
```

ENVIRONMENT VARIABLES
=====================
**SPLINTER_REST_API_URL**
: URL for the `splinterd` REST API. (See `-U`, `--url`.)

SEE ALSO
========
| `splinter-circuit-list(1)`
| `splinter-circuit-disband(1)`
| `splinter-circuit-show(1)`
|
| Splinter documentation: https://www.splinter.dev/docs/0.5/
