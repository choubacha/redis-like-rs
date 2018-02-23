# Redis-like

A naive and simple implementation of in-memory key value store.

### Usage

```
cargo run
```

Then you can telnet or netcat in and try the two commands:

```
$ nc localhost 9000
get hello

set hello world
1
get hello
world
set hello world_2
1
get hello
world_2
kdjf
ERR CommandNotFound

```

The structure of the commands is simple, they must be lower case, they must have their
params separated by a single space, and the command is delimited by a new line.

### Todo

[x] Accept connections
[x] Parse commands
[x] Put commands into a tranaction
[x] Send transaction to a "db"
[x] Listen for results
[x] Write results back to connection

### Structure

There are essentially two actors in the system. A `Connection` and a `Db`. Both
will be run on their own threads so that we can still accept connections even while the db is
is executing commands. The threads are run on async reactors in tokio and communicate via
channels. Every connection will have a channel and will share the `Sender` along with the
transaction. The Db will process the command and send the results back.
