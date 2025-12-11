# Connection handshake protocol
@0xb2c3d4e5f6a7b8c9;

# Version and protocol information for connection handshake
struct VersionDummy {
    # Protocol version magic numbers
    enum Version {
        v01 @0;  # 0x3f61ba36
        v02 @1;  # 0x723081e1 - Authorization key during handshake
        v03 @2;  # 0x5f75e83e - Authorization key and protocol during handshake
        v04 @3;  # 0x400c2d20 - Queries execute in parallel
        v10 @4;  # 0x34c2bdc3 - Users and permissions
    }
    
    # The protocol to use after handshake (specified in V0_3)
    enum Protocol {
        protobuf @0;  # 0x271ffc41
        json @1;      # 0x7e6970c7
    }
    
    version @0 :Int32;
}

# Magic number constants (to be used by client)
const versionV01 :UInt32 = 0x3f61ba36;
const versionV02 :UInt32 = 0x723081e1;
const versionV03 :UInt32 = 0x5f75e83e;
const versionV04 :UInt32 = 0x400c2d20;
const versionV10 :UInt32 = 0x34c2bdc3;

const protocolProtobuf :UInt32 = 0x271ffc41;
const protocolJson :UInt32 = 0x7e6970c7;
