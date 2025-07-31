use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

// Let's manually compute what the hash should be with the new field
// This approximates the IndexingSettings struct with the new max_time_range_secs field

#[derive(Hash)]
struct MockIndexingSettings {
    pub commit_timeout_secs: usize,
    pub docstore_compression_level: i32,
    pub docstore_blocksize: usize,
    pub split_num_docs_target: usize,
    // Assuming MergePolicyConfig and IndexingResources hash consistently
    pub merge_policy_hash: u64,  // placeholder
    pub resources_hash: u64,     // placeholder
    pub max_time_range_secs: Option<u64>,  // NEW FIELD
}

fn main() {
    println!("Simulating hash calculation...");
    
    // Default values from IndexingSettings::default()
    let old_settings = MockIndexingSettings {
        commit_timeout_secs: 60,
        docstore_compression_level: 8,
        docstore_blocksize: 1_000_000,
        split_num_docs_target: 10_000_000,
        merge_policy_hash: 0, // simplified
        resources_hash: 0,    // simplified
        max_time_range_secs: None, // The new field
    };
    
    let mut hasher = SipHasher::new();
    old_settings.hash(&mut hasher);
    let new_hash = hasher.finish();
    
    println!("Simulated new hash with max_time_range_secs field: {}", new_hash);
    
    // The actual difference would be more complex, but this gives us an idea
    // that the hash values will definitely change due to the new field
}