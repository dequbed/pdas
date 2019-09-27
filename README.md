# PDAS

PDAS is a tool to manage large amounts of media files like (e)Books, Music,
pod casts, Videos, Images, ...

PDAS has three main functionalities. Firstly it indexes files, putting all of
their metadata into a read-optimized and indexed database. Secondly it provides
a query interface to allow an user to find media files based on metadata. And
lastly it manages a git-annex repository so that an user can store media files
in a heavily distributed fashion not being required to keep files locally
available to query their metadata.

# Library / Binary

PDAS is split into two parts: The command line interface `pdas` and the support
library `rarian-lib`.

The main reason for this split is so that if in the future somebody wants to
write a different front end (e.g. one based on Qt or a non-interactive one for
scripting) they can directly use `rarian-lib` instead of having to use
calls to `pdas`. This also enforces a clean separation between porcelain
(`pdas`) and plumbing (`rarian-lib`). In general `rarian-lib` should
handle all the actual functionality and the job of a fronted such as
`pdas` should be limited to: 

* Taking (user, system, etc.) input, interpreting it and calling the right
functions in `rarian-lib` 
* Relaying the result of the action to the user/system by printing confirmation/failure notices
* Formatting information returned by `rarian-lib` in the way that makes the most sense in the context (I.e. JSON / Human readable structured formats / debug information to `syslog` / etc.)

Additional support libraries that are useful outside of the limited scope
of PDAS but are still developed as part of PDAS (e.g. the `deterom`
library) are also placed in the lib folder.
