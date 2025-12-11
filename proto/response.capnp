# Response message protocol
@0xe5f6a7b8c9daebfc;

using Types = import "types.capnp";

# Response types
enum ResponseType {
    # Success responses
    successAtom @0;      # Single datum response
    successSequence @1;  # Complete sequence of datums
    successPartial @2;   # Partial sequence (more data available)
    waitComplete @3;     # NOREPLY_WAIT completed
    serverInfo @4;       # Server information
    
    # Error responses
    clientError @5;      # Client is buggy (malformed request)
    compileError @6;     # Query failed during parsing/type checking
    runtimeError @7;     # Query failed at runtime
}

# Error types (for RUNTIME_ERROR responses)
enum ErrorType {
    internal @0;         # 1000000 - Internal RethinkDB error
    resourceLimit @1;    # 2000000 - Resource limit exceeded
    queryLogic @2;       # 3000000 - Query logic error
    nonExistence @3;     # 3100000 - Missing value
    opFailed @4;         # 4100000 - Operation failed
    opIndeterminate @5;  # 4200000 - Operation indeterminate
    user @6;             # 5000000 - User error
    permissionError @7;  # 6000000 - Permission denied
}

# Response notes (special properties of streams)
enum ResponseNote {
    sequenceFeed @0;      # Stream is a changefeed
    atomFeed @1;          # Stream is a single-document feed
    orderByLimitFeed @2;  # Stream is an order_by_limit feed
    unionedFeed @3;       # Stream is a union of multiple feed types
    includesStates @4;    # Stream includes state notifications
}

# Response message sent from server to client
struct Response {
    type @0 :ResponseType;
    
    # Response token (matches query token)
    token @1 :Int64;
    
    # Response data (datums or error message)
    response @2 :List(Types.Datum);
    
    # Backtrace (for errors)
    backtrace @3 :Types.Backtrace;
    
    # Profile information (if requested)
    profile @4 :Types.Datum;
    
    # Error type (for error responses)
    errorType @5 :ErrorType;
    
    # Response notes
    notes @6 :List(ResponseNote);
}
