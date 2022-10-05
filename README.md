
# NSO log reader

Read NSO logs with ease and comfort.

## TODO

[ ] Multi-line log messages are not output if they are the last message in the
    stream, since the parser is still waiting for more lines. Solve this by
    adding a short timer. All log messages are written to the log output in one
    go, i.e. a single log message cannot be streamed, so any delay means that the
    log message is done.

[x] Format the time as local time rather than UTC

[ ] Any time a log message is printed, check how long has passed since the
    previous log message. If it's longer than X seconds, first print a separator
    that looks something like this:

    --- 15 seconds later -------------------------------------------------------

    Perhaps make it cover the full terminal width?

[ ] Allow filtering by severity, that way the user can keep debug logging
    enabled but only have to see them when needed.
