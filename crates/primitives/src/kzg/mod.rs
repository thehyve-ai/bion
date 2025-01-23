mod coefficient;
mod kzg_commitment;
mod kzg_proof;

pub use self::coefficient::*;
pub use kzg_commitment::*;
pub use kzg_proof::*;

use kzg_traits::eip_4844::BYTES_PER_FIELD_ELEMENT;

/// Calculates the encoded blob length (B) after erasure encoding.
/// Ensures that the encoded length satisfies the quorum and adversary thresholds.
///
/// # Arguments
/// * `blob_length` - The length of the blob **in symbols**
/// * `adversary_threshold` - The adversary threshold as an integer percentage (e.g., 80 for 0.8).
/// * `quorum_threshold` - The quorum threshold as an integer percentage (e.g., 90 for 0.9).
///
/// # Returns
/// The encoded blob length (B) as an integer.
pub fn calculate_encoded_blob_length(
    blob_length: u64,
    adversary_threshold: u32,
    quorum_threshold: u32,
) -> u64 {
    if quorum_threshold <= adversary_threshold {
        panic!("Beta must be greater than Alpha to ensure valid thresholds.");
    }

    let y = (quorum_threshold - adversary_threshold) as u64;
    (blob_length * 100 + y - 1) / (y)
}

/// Calculates the chunk size (C) for each node.
/// Ensures the chunk size satisfies the distribution requirements for the network.
///
/// # Arguments
/// * `blob_length` - The size of the original blob (in symbols).
/// * `num_nodes` - The number of nodes in the network.
/// * `alpha` - The adversary threshold as an integer percentage (e.g., 80 for 0.8).
/// * `beta` - The quorum threshold as an integer percentage (e.g., 90 for 0.9).
///
/// # Returns
/// The chunk size (C) as an integer.
pub fn calculate_chunk_length(
    blob_length: u64,
    minimum_chunk_length: u64,
    num_nodes: u64,
    adversary_threshold: u32,
    quorum_threshold: u32,
) -> u64 {
    let mut chunk_length = minimum_chunk_length;

    loop {
        let is_ok = {
            let total_encoded = chunk_length * num_nodes;

            let required_encoded_length = (blob_length * 100
                + ((quorum_threshold as u64) - (adversary_threshold as u64))
                - 1)
                / ((quorum_threshold as u64) - (adversary_threshold as u64));

            // Check if the total encoded length is sufficient
            total_encoded >= required_encoded_length
        };

        if is_ok {
            return chunk_length;
        }

        chunk_length *= 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodingParameters {
    pub num_samples: u64,
    pub sample_length: u64,
}

impl EncodingParameters {
    pub fn new(num_samples: u64, sample_length: u64) -> Self {
        Self {
            num_samples,
            sample_length,
        }
    }

    pub fn from_minimal(min_samples: u64, min_sample_length: u64) -> Self {
        Self {
            num_samples: min_samples.next_power_of_two(),
            sample_length: min_sample_length.next_power_of_two(),
        }
    }

    pub fn from_parities(num_system: usize, num_parities: usize, data_size: usize) -> Self {
        let num_nodes = num_system + num_parities;
        let number_of_symbols = (data_size + BYTES_PER_FIELD_ELEMENT - 1) / BYTES_PER_FIELD_ELEMENT;
        let chunk_len = (number_of_symbols + num_system - 1) / num_system;

        Self::from_minimal(num_nodes as u64, chunk_len as u64)
    }

    pub fn point_evaluations(&self) -> usize {
        (self.num_samples * self.sample_length) as usize
    }

    pub fn from_parameters(
        blob_length: u64,
        minimum_chunk_length: u64,
        num_nodes: u64,
        adversary_threshold: u32,
        quorum_threshold: u32,
    ) -> Self {
        debug_assert!(
            num_nodes.is_power_of_two(),
            "Num nodes needs to a power of 2"
        );
        let chunk_length = calculate_chunk_length(
            blob_length,
            minimum_chunk_length,
            num_nodes,
            adversary_threshold,
            quorum_threshold,
        );
        let num_samples = num_nodes;
        let sample_length = chunk_length;

        Self {
            num_samples,
            sample_length,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calculate_chunk_size_constraints() {
        let adversary_thresholds = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let quorum_thresholds = vec![15, 25, 35, 45, 55, 65, 75, 85, 95];
        let num_nodes = vec![16, 32, 64, 128, 256, 512, 1024, 2048];
        let blob_lengths = vec![50, 64, 72, 82, 96, 102, 117, 625, 983, 1000, 8572];

        // test all combinations

        for &adversary_threshold in adversary_thresholds.iter() {
            for &quorum_threshold in quorum_thresholds.iter() {
                if quorum_threshold <= adversary_threshold {
                    continue;
                }
                for &num_node in num_nodes.iter() {
                    for &blob_length in blob_lengths.iter() {
                        let chunk_size = calculate_chunk_length(
                            blob_length,
                            2,
                            num_node,
                            adversary_threshold,
                            quorum_threshold,
                        );
                        let encoded_blob_length = calculate_encoded_blob_length(
                            blob_length,
                            adversary_threshold,
                            quorum_threshold,
                        );

                        // chunk_size = power of 2
                        assert_eq!(chunk_size.count_ones(), 1, "chunk_size = power of 2");

                        let coding_rate = blob_length * 100 / ((num_node as u64) * chunk_size);
                        let y = (quorum_threshold - adversary_threshold) as u64;

                        assert!(coding_rate <= y);

                        let calculated_encoded_length = chunk_size * (num_node as u64);

                        assert!(calculated_encoded_length >= encoded_blob_length);
                    }
                }
            }
        }
    }
}
