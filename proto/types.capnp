# Common types used across RethinkDB protocol
@0xa1b2c3d4e5f6a7b8;

# A Datum is a chunk of data that can be serialized to disk or returned to
# the user in a Response. Currently we only support JSON types.
struct Datum {
    union {
        null @0 :Void;
        bool @1 :Bool;
        number @2 :Float64;
        string @3 :Text;
        array @4 :List(Datum);
        object @5 :List(AssocPair);
        # JSON encoding of the Datum (optimization for clients)
        json @6 :Text;
    }
}

# Key-value pair for objects
struct AssocPair {
    key @0 :Text;
    value @1 :Datum;
}

# Backtrace frame (for error reporting)
struct Frame {
    enum FrameType {
        pos @0;  # Error in positional argument
        opt @1;  # Error in optional argument
    }
    type @0 :FrameType;
    pos @1 :Int64;   # Index of positional argument
    opt @2 :Text;    # Name of optional argument
}

struct Backtrace {
    frames @0 :List(Frame);
}
