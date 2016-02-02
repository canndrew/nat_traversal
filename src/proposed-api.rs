/// The address of a server that can be used to obtain an external address.
pub enum HolePunchServerAddr {
    /// A server which speaks the simple hole punching protocol (ie. our MaidSafe protocol). This
    /// should probably be deprecated and replaced with a proper STUN implementation.
    Simple(SocketAddrV4),
    /// An Internet Gateway Device that can be used for UPnP port mapping.
    IgdGateway(igd::Gateway)
}

/// You need to create a `MappingContext` before doing any socket mapping. This `MappingContext`
/// should ideally be kept throughout the lifetime of the program. Internally it caches a
/// addresses of UPnP servers and hole punching servers.
struct MappingContext {
    servers: RwLock<Vec<HolePunchServerAddr>>,
}

impl MappingContext {
    /// Create a new mapping context. This will block breifly while it searches the network for
    /// UPnP servers.
    fn new() -> MappingContext,

    /// Inform the context about external hole punching servers.
    fn add_servers<S>(&self, servers: S)
        where S: IntoIterator<Item=HolePunchServerAddr>
}

/// A socket address obtained through some mapping technique.
pub struct MappedSocketAddr {
    /// The mapped address
    pub addr: SocketAddrV4,

    /// Indicated that hole punching needs to be used for an external client to connect to this
    /// address. `nat_restricted` will not be set if this is a fully mapped address such as the
    /// external address of a full-cone NAT or one obtained through UPnP.
    pub nat_restricted: bool,
}

/// Info needed by both parties when performing a rendezvous connection.
struct RendezvousInfo {
    /// A vector of all the mapped addresses that the peer can try connecting to.
    endpoints: Vec<MappedSocketAddr>,
    /// Used to identify the peer.
    secret: [u8; 4],
}

impl RendezvousInfo {
    /// Create rendezvous info for being sent to the remote peer.
    pub fn from_endpoints(endpoints: Vec<MappedSocketAddr>) -> RendezvousInfo;
}

/// A bound udp socket for which we know our external endpoints.
struct MappedUdpSocket {
    /// The socket.
    pub socket: UdpSocket,
    /// The known endpoints of this socket.
    pub endpoints: Vec<MappedSocketAddr>
}

impl MappedUdpSocket {
    /// Map an existing `UdpSocket`.
    pub fn map(socket: UdpSocket, mc: &MappingContext)
        -> MappedUdpSocket

    /// Create a new `MappedUdpSocket`
    pub fn new(mc: &MappingContext)
        -> MappedUdpSocket
}

/// A udp socket that has been hole punched.
struct PunchedUdpSocket {
    pub socket: UdpSocket,
    pub peer_addr: SocketAddr,
}

impl PunchedUdpSocket {
    /// Punch a udp socket using a mapped socket and the peer's rendezvous info.
    pub fn punch_hole(socket: UdpSocket, their_rendezvous_info: RendezvousInfo)
        -> PunchedUdpSocket
}

/// A tcp socket for which we know our external endpoints.
struct MappedTcpSocket {
    /// A bound, but neither listening or connected tcp socket. The socket is bound to be reuseable
    /// (ie. SO_REUSEADDR is set as is SO_REUSEPORT on unix).
    pub socket: net2::TcpBuilder,
    /// The known endpoints of this socket.
    pub endpoints: Vec<MappedSocketAddr>,
}

impl MappedTcpSocket {
    /// Map an existing tcp socket. The socket must not bound or connected. This function will set
    /// the options to make the socket address reuseable before binding it.
    pub fn map(socket: net2::TcpBuilder, mc: &MappingContext)
        -> MappedTcpSocket;

    /// Create a new `MappedTcpSocket`
    pub fn new(mc: &MappingContext)
        -> MappedTcpSocket;
}

/// Perform a tcp rendezvous connect. `socket` should have been obtained from a `MappedTcpSocket`.
pub fn tcp_punch_hole(socket: net2::TcpBuilder, their_rendezvous_info: RendezvousInfo)
    -> TcpStream;

/// RAII type for a hole punch server which speaks the simple hole punching protocol.
struct SimpleUdpHolePunchServer<'a> {
    mapping_context: &'a MappingContext,
}

impl<'a> SimpleUdpHolePunchServer<'a> {
    /// Create a new server. This will spawn a background thread which will serve requests until
    /// the server is dropped.
    pub fn new(mapping_context: &'a MappingContext)
        -> SimpleUdpHolePunchServer<'a>;

    /// Get the external addresses of this server to be shared with peers.
    pub fn addresses(&self)
        -> Vec<MappedSocketAddr>
}
