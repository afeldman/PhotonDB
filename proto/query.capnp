# Query message protocol
@0xd4e5f6a7b8c9daeb;

using Term = import "term.capnp";
using Types = import "types.capnp";

# Query types
enum QueryType {
    start @0;       # Start a new query
    continue @1;    # Continue a query that returned SUCCESS_PARTIAL
    stop @2;        # Stop a query partway through executing
    noreplyWait @3; # Wait for noreply operations to finish
    serverInfo @4;  # Get server information
}

# Query message sent from client to server
struct Query {
    type @0 :QueryType;
    
    # Query term (for START queries)
    query @1 :Term.Term;
    
    # Token for tracking async queries
    token @2 :Int64;
    
    # OBSOLETE: Use global optargs instead
    obsoleteNoreply @3 :Bool = false;
    
    # If true, Datum values may be R_JSON for speedups
    acceptsRJson @4 :Bool = false;
    
    # Global optional arguments
    globalOptargs @5 :List(Types.AssocPair);
}
