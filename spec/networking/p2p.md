# Network Overview

This section outlines the high level overview of the network specification

## Encryption

The same as Ethereum consensus clients, we implement [Libp2p-noise](https://github.com/libp2p/specs/tree/master/noise) secure channel handshake with `secp256k1`. This makes logical sense since we are an extension of Ethereum and use Keypairs to identify nodes.

As specified in the libp2p specification, clients MUST support the `XX` handshake pattern.

## Transport

The implementation MUST support QUIC libp2p transport, and it MUST be enabled for both dialing and listening. I believe libp2p QUIC transport only supports IPv4 addressees.

Clients MUST support listening and dialing on IPv4.

## Protocol Negotiation

Clients MUST use exact equality when negotiating protocol versions to use and MAY use the version to give priority to higher version numbers.

# Network Interactions

## Types

## Data sharing

### Protocol Identification

The p2p network of HyveDA has multiple substreams that is specific to the purpose of the message. Each substream is a Request-Response

The protocol ID is case sensitive UTF-8 String:

`/ProtocolPrefix/DACIdentifier/MessageName/SchemaVersion/Encoding`

- `ProtocolPrefix` - messages are grouped under a shared protocol prefix. In the current functionality it's `/hda/data_availability/req`
- `DACIdentifier` - the identifier of the Data Availability Committee. At genesis, this can only be `0`
- `MessageName` - Describes the message type
- `SchemaVersion` - The version number of the schema used (e.g., 1.0.0).
- `Encoding` - The schema defines the data types in abstract terms, the encoding describes the representation of bytes that is transmitted over the wire. These are similar to Ethereum beacon chain: `ssz_snappy`

### messages

#### chunks_transfer

Request content:

```
{
	blob_header: {
		commitment: KZGCommitment
		length: BlobSize
		app_id: AppID,
	}
	chunks: Vector<{
		commitment: KZGCommitment
		proof: KZGProof
		data: Bytes
	}>
}
```

Response content:

```
{
	ack: Vector<{
		chunk_index: u32,
		status: "ok" | "error"
		error_code: 0 | 1 | 2 | ...
	}>
}
```

### FAQ

- The same P2P will be used for all nodes, how do we get the connections only for the capabilities
