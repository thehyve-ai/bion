# Discovery spec

# Technical Specification for Implementing a Kademlia DHT

This technical specification details the components, algorithms, and steps necessary to implement a Kademlia-based Distributed Hash Table (DHT). The document outlines the core principles, system design, and protocols involved in implementing Kademlia, which is a P2P (peer-to-peer) system widely used in decentralized networks.

---

## 1. Overview

The Kademlia DHT provides a decentralized key-value store with efficient node lookups using a structured overlay network. Each node and key in the network has a unique identifier (node ID and key ID), and routing is based on the XOR distance between these identifiers.

The primary goals of Kademlia include:

- Efficient lookup of nodes and data.
- Scalability and fault tolerance.
- Tolerance to high churn (nodes frequently joining and leaving).

Kademlia achieves these goals using the XOR metric for distance calculation and iterative lookups over a structured routing table with "K-buckets."

---

## 2. Core Components

### 2.1 Replication parameter (k)

- The amount of replication is governed by the replication parameter k. The recommended value for k is 20.

### 2.2 Distance Metric

- Kademlia uses the **XOR distance** between two node IDs (or between a node ID and a key ID) to determine proximity.
- The XOR metric: In all cases, the distance between two keys is XOR(sha256(key1), sha256(key2)).

### 2.3 Alpha concurrency parameter (α)

- The concurrency of node and value lookups are limited by parameter α, with a default value of 3.
- This implies that each lookup process can perform no more than 3 inflight requests, at any given time.

### 2.4 Peer ID

- Each node in the Kademlia network is identified by a unique **peer ID**.
- Peer IDs are derived by hashing the encoded public key with multihash.
- Peer Ids always use the base58 encoding, with no multibase prefix when encoded into strings

### 2.5 Key

- Keys in the DHT keyspace identify both the participating nodes, as well as the records stored in the DHT.
- Keys have an XOR metric as defined in the Kademlia paper, i.e. the bitwise XOR of the hash digests, interpreted as an integer

---

## 3. Routing Table

Each node maintains a **routing table** of known peers in the network, structured into **K-buckets**. Each K-bucket contains up to `K` nodes that are progressively farther away from the node in XOR space.

### 3.1 K-buckets

- The routing table is divided into **K-buckets** based on the XOR distance from the node's own ID.
- A K-bucket holds nodes with XOR distances that fall into specific ranges:
- **Bucket 0**: Nodes with XOR distance in the range `[2^0, 2^1)`
- **Bucket 1**: Nodes with XOR distance in the range `[2^1, 2^2)`
- ...
- **Bucket N**: Nodes with XOR distance `[2^N, 2^(N+1))`

- Each K-bucket can store up to **K** nodes (a small constant, typically `K=20`).

### 3.2 Least Recently Seen (LRS) Replacement

- Nodes in each K-bucket are ordered by the time of their last interaction.
- When a K-bucket is full, Kademlia employs a **Least Recently Seen** policy:
- The least recently seen node is **pinged** to check if it's still alive.
- If the node does not respond, it is replaced by the new node.
- If it does respond, the new node is discarded unless it’s in a closer K-bucket.

---

## 4. DHT Operations

### 4.1 Join Operation (Bootstrap)

When a new node joins the Kademlia network, it must discover existing nodes to populate its routing table. This is done via **bootstrap nodes** that are known to be online.

**Steps:**

1. **Find Bootstrap Nodes:** The new peer contacts a known bootstrap node.
2. **Initial Lookup:** The new peer performs a **FIND_NODE** operation for its own peer ID.

- The bootstrap node returns a list of peers close to the new peer's ID.

3. **Routing Table Population:** The new node uses the list returned from the lookup to query additional nodes, filling its routing table.

### 4.2 Peer routing

- Finding the closest node via **FIND_NODE**

The **FIND_NODE** operation retrieves nodes closest to a given target ID (node ID or key ID).

**Steps:**

1. The querying node sends a **FIND_NODE** request to the K closest nodes in its routing table.
2. Each queried node responds with the K nodes closest to the target ID from their own routing tables.
3. The querying node repeats the process, contacting nodes that are progressively closer to the target ID until the K closest nodes are found.

### 4.3 Value storage and retrieval

- Storing a value on the nodes closest to the value's key by looking up the closest nodes via **FIND_NODE** and then putting the value to those nodes via **PUT_VALUE**.

The **STORE** operation allows a node to save a value associated with a key into the network. Nodes store data in peers closest to the key’s ID.

**Steps:**

1. The node computes the key ID (hash of the data or identifier).
2. The node performs a **FIND_NODE** lookup to find the K closest nodes to the key ID.
3. The node sends a **PUT_VALUE** message to each of the closest K nodes, along with the key-value pair.

- Getting a value by its key from the nodes closest to that key via **GET_VALUE**.

The **GET_VALUE** operation retrieves the value associated with a given key from the network.

**Steps:**

1. The querying node computes the key ID.
2. The node performs a **FIND_NODE** lookup for the key ID.
3. If any of the K closest nodes have the value stored, they return it.
4. If no node has the value, the closest nodes to the key ID are returned instead, allowing the node to search further.

### 4.4 Content provider advertisement and discovery

- Adding oneself to the list of providers for a given key at the nodes closest to that key by finding the closest nodes via **FIND_NODE** and then adding oneself via **ADD_PROVIDER**

- Getting providers for a given key from the nodes closest to that key via **GET_PROVIDERS**

---

## 6. Kademlia Protocol

### 6.1 Message Types

The following message types are core to Kademlia communication:

1. **FIND_NODE:**

- In the request **key** must be set to the binary **PeerId** of the node to be found. In the response **closerPeers** is set to the **k** closest peers.

2. **GET_VALUE**

- In the request **key** is an unstructured array of bytes. **record** is set to the value for the given key (if found in the datastore) and **closerPeers** is set to the **k** closest peers.

3. **PUT_VALUE**

- In the request **record** is set to the record to be stored and **key** on **Message** is set to equal **key** of the **Record**. The target node validates **record**, and if it is valid, it stores it in the datastore and as a response echoes the request.

4. **GET_PROVIDERS**

- In the request **key** is set to a CID. The target node returns the closest known **providerPeers** (if any) and the **k** closest known **closerPeers**.

5. **ADD_PROVIDER**

- In the request **key** is set to a CID. The target node verifies **key** is a valid CID, all **providerPeers** that match the RPC sender's PeerID are recorded as providers.

6. **PING:**

- Deprecated message type replaced by the dedicated ping protocol. Implementations may still handle incoming PING requests for backwards compatibility. Implementations must not actively send PING requests.

### 6.2 Protocol Flow Example (FIND_NODE)

1. Node A wants to find the K closest nodes to a target ID.
2. Node A consults its routing table for the K closest nodes and sends **FIND_NODE** requests.
3. Each recipient node returns its closest known nodes.
4. Node A iterates by querying the returned nodes, continuing until it reaches nodes that are close enough (convergence).

---

## 7. Network Maintenance

### Bootstrap process

- The bootstrap process is responsible for keeping the routing table filled and healthy throughout time. The below is one possible algorithm to bootstrap.

- The process runs once on startup, then periodically with a configurable frequency (default: 5 minutes). On every run, we generate a random **peer ID** and we look it up via the process defined in **Peer Routing**. Peers encountered throughout the search are inserted in the routing table, as per usual.

- This is repeated as many times per run as configuration parameter **QueryCount** (default: 1). In addition, to improve awareness of nodes close to oneself, implementations should include a lookup for their own **peer ID**.

- Every repetition is subject to a **QueryTimeout** (default: 10 seconds), which upon firing, aborts the run.

### 7.2 Node Churn

Nodes may leave or join the network frequently (churn). Kademlia handles churn efficiently via:

- **Pinging nodes:** Checking the liveness of nodes in K-buckets.
- **Bucket management:** Replacing inactive nodes with new ones discovered during lookups.
- **Replication:** Redundantly storing key-value pairs across multiple nodes to mitigate data loss from churn.

---

## 8. Concurrency and Parallelism

Kademlia supports **concurrent** lookups by allowing a nodee to query multiple peers simultaneously during operations like **FIND_NODE** and **GET_VALUE**.

- **α-parameter:** Kademlia introduces an `α` parameter that defines the degree of parallelism (typically `α=3`). This means that during a lookup, a node will query `α` nodes in parallel.

---
