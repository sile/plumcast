//! [Prometheus][prometheus] metrics.
//!
//! Note that you can also use [fibers_rpc's metrics] in addition to the metrics defined in this module.
//!
//! [prometheus]: https://prometheus.io/
//! [fibers_rpc's metrics]: https://docs.rs/fibers_rpc/0.2/fibers_rpc/metrics/index.html
use prometrics::metrics::{Counter, MetricBuilder};

/// Metrics of a [`Service`].
///
/// [`Service`]: ../struct.Service.html
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    pub(crate) registered_nodes: Counter,
    pub(crate) deregistered_nodes: Counter,
    pub(crate) destination_unknown_messages: Counter,
}
impl ServiceMetrics {
    /// Metric: `plumcast_service_registered_nodes_total <COUNTER>`
    pub fn registered_nodes(&self) -> u64 {
        self.registered_nodes.value() as u64
    }

    /// Metric: `plumcast_service_deregistered_nodes_total <COUNTER>`
    pub fn deregistered_nodes(&self) -> u64 {
        self.deregistered_nodes.value() as u64
    }

    /// Metric: `plumcast_service_destination_unknown_messages_total <COUNTER>`
    pub fn destination_unknown_messages(&self) -> u64 {
        self.destination_unknown_messages.value() as u64
    }

    pub(crate) fn new(mut builder: MetricBuilder) -> Self {
        builder.namespace("plumcast").subsystem("service");
        ServiceMetrics {
            registered_nodes: builder
                .counter("registered_nodes_total")
                .help("Number of nodes registered so far")
                .finish()
                .expect("Never fails"),
            deregistered_nodes: builder
                .counter("deregistered_nodes_total")
                .help("Number of nodes deregistered so far")
                .finish()
                .expect("Never fails"),
            destination_unknown_messages: builder
                .counter("destination_unknown_messages_total")
                .help("Number of RPC messages received but the destination node is missing")
                .finish()
                .expect("Never fails"),
        }
    }
}

/// Metrics of a [`Node`].
///
/// [`Node`]: ../struct.Node.html
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub(crate) broadcasted_messages: Counter,
    pub(crate) forgot_messages: Counter,
    pub(crate) delivered_messages: Counter,
    pub(crate) connected_neighbors: Counter,
    pub(crate) disconnected_neighbors: Counter,
    pub(crate) isolated_times: Counter,
    pub(crate) deisolated_times: Counter,
    pub(crate) forget_unknown_message_errors: Counter,
    pub(crate) cannot_send_hyparview_message_errors: Counter,
    pub(crate) cannot_send_plumtree_message_errors: Counter,
    pub(crate) unknown_plumtree_node_errors: Counter,
}
impl NodeMetrics {
    /// Metric: `plumcast_node_broadcasted_messages_total <COUNTER>`
    pub fn broadcasted_messages(&self) -> u64 {
        self.broadcasted_messages.value() as u64
    }

    /// Metric: `plumcast_node_forgot_messages_total <COUNTER>`
    pub fn forgot_messages(&self) -> u64 {
        self.forgot_messages.value() as u64
    }

    /// Metric: `plumcast_node_delivered_messages_total <COUNTER>`
    pub fn delivered_messages(&self) -> u64 {
        self.delivered_messages.value() as u64
    }

    /// Metric: `plumcast_node_connected_neighbors_total <COUNTER>`
    pub fn connected_neighbors(&self) -> u64 {
        self.connected_neighbors.value() as u64
    }

    /// Metric: `plumcast_node_disconnected_neighbors_total <COUNTER>`
    pub fn disconnected_neighbors(&self) -> u64 {
        self.disconnected_neighbors.value() as u64
    }

    /// Metric: `plumcast_node_isolated_times_total <COUNTER>`
    pub fn isolated_times(&self) -> u64 {
        self.isolated_times.value() as u64
    }

    /// Metric: `plumcast_node_deisolated_times_total <COUNTER>`
    pub fn deisolated_times(&self) -> u64 {
        self.deisolated_times.value() as u64
    }

    /// Metric: `plumcast_node_errors_total { kind="forget_unknown_message" } <COUNTER>`
    pub fn forget_unknown_message_errors(&self) -> u64 {
        self.forget_unknown_message_errors.value() as u64
    }

    /// Metric: `plumcast_node_errors_total { kind="cannot_send_hyparview_message" } <COUNTER>`
    pub fn cannot_send_hyparview_message_errors(&self) -> u64 {
        self.cannot_send_hyparview_message_errors.value() as u64
    }

    /// Metric: `plumcast_node_errors_total { kind="cannot_send_plumtree_message" } <COUNTER>`
    pub fn cannot_send_plumtree_message_errors(&self) -> u64 {
        self.cannot_send_plumtree_message_errors.value() as u64
    }

    /// Metric: `plumcast_node_errors_total { kind="unknown_plumtree_node" } <COUNTER>`
    pub fn unknown_plumtree_node_errors(&self) -> u64 {
        self.unknown_plumtree_node_errors.value() as u64
    }

    pub(crate) fn new(mut builder: MetricBuilder) -> Self {
        builder.namespace("plumcast").subsystem("node");
        NodeMetrics {
            broadcasted_messages: builder
                .counter("broadcasted_messages_total")
                .help("Number of messages broadcasted so far")
                .finish()
                .expect("Never fails"),
            forgot_messages: builder
                .counter("forgot_messages_total")
                .help("Number of messages forgot so far")
                .finish()
                .expect("Never fails"),
            delivered_messages: builder
                .counter("delivered_messages_total")
                .help("Number of messages delivered so far")
                .finish()
                .expect("Never fails"),
            connected_neighbors: builder
                .counter("connected_neighbors_total")
                .help("Number of neighbors connected so far")
                .finish()
                .expect("Never fails"),
            disconnected_neighbors: builder
                .counter("disconnected_neighbors_total")
                .help("Number of neighbors disconnected so far")
                .finish()
                .expect("Never fails"),
            isolated_times: builder
                .counter("isolated_times_total")
                .help("Number of times the node was isolated so far")
                .finish()
                .expect("Never fails"),
            deisolated_times: builder
                .counter("deisolated_times_total")
                .help("Number of times the node was de-isolated so far")
                .finish()
                .expect("Never fails"),
            forget_unknown_message_errors: builder
                .counter("errors_total")
                .help("Number of errors happened so far")
                .label("kind", "forget_unknown_message")
                .finish()
                .expect("Never fails"),
            cannot_send_hyparview_message_errors: builder
                .counter("errors_total")
                .help("Number of errors happened so far")
                .label("kind", "cannot_send_hyparview_message")
                .finish()
                .expect("Never fails"),
            cannot_send_plumtree_message_errors: builder
                .counter("errors_total")
                .help("Number of errors happened so far")
                .label("kind", "cannot_send_plumtree_message")
                .finish()
                .expect("Never fails"),
            unknown_plumtree_node_errors: builder
                .counter("errors_total")
                .help("Number of errors happened so far")
                .label("kind", "unknown_plumtree_node")
                .finish()
                .expect("Never fails"),
        }
    }

    pub(crate) fn add(&self, other: &Self) {
        self.broadcasted_messages
            .add_u64(other.broadcasted_messages());
        self.forgot_messages.add_u64(other.forgot_messages());
        self.delivered_messages.add_u64(other.delivered_messages());
        self.connected_neighbors
            .add_u64(other.connected_neighbors());
        self.disconnected_neighbors
            .add_u64(other.disconnected_neighbors());
        self.isolated_times.add_u64(other.isolated_times());
        self.deisolated_times.add_u64(other.deisolated_times());
        self.forget_unknown_message_errors
            .add_u64(other.forget_unknown_message_errors());
        self.cannot_send_hyparview_message_errors
            .add_u64(other.cannot_send_hyparview_message_errors());
        self.cannot_send_plumtree_message_errors
            .add_u64(other.cannot_send_plumtree_message_errors());
        self.unknown_plumtree_node_errors
            .add_u64(other.unknown_plumtree_node_errors());
    }
}
