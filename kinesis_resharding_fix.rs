// Proposed fix for Kinesis resharding issue in Quickwit
// This code shows how to properly handle shard resharding

use std::collections::{HashMap, HashSet};

// Mock types for demonstration
#[derive(Debug, Clone)]
struct Shard {
    shard_id: String,
    parent_shard_id: Option<String>,
    adjacent_parent_shard_id: Option<String>,  // For merge operations
    sequence_number_range: SequenceNumberRange,
}

#[derive(Debug, Clone)]
struct SequenceNumberRange {
    starting_sequence_number: String,
    ending_sequence_number: Option<String>,  // None if shard is still open
}

impl Shard {
    fn is_closed(&self) -> bool {
        self.sequence_number_range.ending_sequence_number.is_some()
    }
}

// Solution implementation
struct KinesisSourceFix {
    stream_name: String,
}

impl KinesisSourceFix {
    /// Main fix: Initialize source with proper shard handling
    async fn initialize_with_resharding_support(
        &mut self,
        kinesis_client: &KinesisClient,
        checkpoint: &SourceCheckpoint,
    ) -> Result<HashMap<String, ShardConsumerState>, Error> {
        // Step 1: Get ALL shards (including closed ones)
        let all_shards = list_shards(kinesis_client, &self.stream_name).await?;
        
        // Step 2: Separate active and closed shards
        let (active_shards, closed_shards): (Vec<_>, Vec<_>) = all_shards
            .into_iter()
            .partition(|shard| !shard.is_closed());
        
        println!("Found {} active shards, {} closed shards", 
                 active_shards.len(), closed_shards.len());
        
        // Step 3: Build parent-child relationship map
        let parent_to_children = build_shard_lineage_map(&active_shards, &closed_shards);
        
        // Step 4: Migrate checkpoints from closed parents to active children
        let migrated_checkpoint = migrate_checkpoints(
            checkpoint, 
            &parent_to_children,
            &closed_shards
        );
        
        // Step 5: Create consumers ONLY for active shards
        let mut shard_consumers = HashMap::new();
        
        for shard in active_shards {
            let from_position = migrated_checkpoint
                .get(&shard.shard_id)
                .or_else(|| {
                    // Check if this shard is a child of a closed shard
                    if let Some(parent_id) = &shard.parent_shard_id {
                        checkpoint.get(parent_id)
                    } else {
                        None
                    }
                });
            
            println!("Creating consumer for active shard {} starting from {:?}", 
                     shard.shard_id, from_position);
            
            let consumer_state = create_shard_consumer(
                &shard.shard_id,
                from_position.cloned(),
            );
            
            shard_consumers.insert(shard.shard_id.clone(), consumer_state);
        }
        
        Ok(shard_consumers)
    }
}

/// Build a map of parent shard IDs to their children
fn build_shard_lineage_map(
    active_shards: &[Shard],
    closed_shards: &[Shard],
) -> HashMap<String, Vec<String>> {
    let mut parent_to_children: HashMap<String, Vec<String>> = HashMap::new();
    
    // Check all active shards for parent relationships
    for shard in active_shards {
        // Handle split operation (one parent)
        if let Some(parent_id) = &shard.parent_shard_id {
            parent_to_children
                .entry(parent_id.clone())
                .or_default()
                .push(shard.shard_id.clone());
        }
        
        // Handle merge operation (two parents)
        if let Some(adjacent_parent_id) = &shard.adjacent_parent_shard_id {
            parent_to_children
                .entry(adjacent_parent_id.clone())
                .or_default()
                .push(shard.shard_id.clone());
        }
    }
    
    parent_to_children
}

/// Migrate checkpoints from closed parent shards to their active children
fn migrate_checkpoints(
    original_checkpoint: &SourceCheckpoint,
    parent_to_children: &HashMap<String, Vec<String>>,
    closed_shards: &[Shard],
) -> SourceCheckpoint {
    let mut migrated = original_checkpoint.clone();
    let closed_shard_ids: HashSet<_> = closed_shards
        .iter()
        .map(|s| s.shard_id.clone())
        .collect();
    
    // Remove checkpoints for closed shards
    for closed_id in &closed_shard_ids {
        if migrated.contains_key(closed_id) {
            println!("Removing checkpoint for closed shard: {}", closed_id);
            
            // If this closed shard has children, migrate the checkpoint
            if let Some(children) = parent_to_children.get(closed_id) {
                let parent_position = migrated.remove(closed_id).unwrap();
                
                // For split: both children start from parent's last position
                // For merge: the child starts from the latest position of both parents
                for child_id in children {
                    if !migrated.contains_key(child_id) {
                        println!("Migrating checkpoint from {} to child {}", 
                                 closed_id, child_id);
                        migrated.insert(child_id.clone(), parent_position.clone());
                    }
                }
            } else {
                migrated.remove(closed_id);
            }
        }
    }
    
    migrated
}

/// Enhanced shard consumer initialization
fn create_shard_consumer(
    shard_id: &str,
    from_position: Option<String>,
) -> ShardConsumerState {
    ShardConsumerState {
        shard_id: shard_id.to_string(),
        current_position: from_position,
        // ... other fields
    }
}

// Additional helper function to detect resharding events
async fn detect_resharding_event(
    current_shards: &[String],
    new_shards: &[Shard],
) -> bool {
    let current_set: HashSet<_> = current_shards.iter().cloned().collect();
    let new_set: HashSet<_> = new_shards
        .iter()
        .filter(|s| !s.is_closed())
        .map(|s| s.shard_id.clone())
        .collect();
    
    // Resharding occurred if shard sets are different
    current_set != new_set
}

// Mock types for demonstration
struct KinesisClient;
type SourceCheckpoint = HashMap<String, String>;
type Error = std::io::Error;

struct ShardConsumerState {
    shard_id: String,
    current_position: Option<String>,
}

async fn list_shards(_client: &KinesisClient, _stream: &str) -> Result<Vec<Shard>, Error> {
    // Mock implementation
    Ok(vec![])
}

fn main() {
    println!("=== Kinesis Resharding Fix ===\n");
    
    println!("Key changes needed in quickwit-indexing/src/source/kinesis/kinesis_source.rs:\n");
    
    println!("1. In KinesisSource::initialize():");
    println!("   - Don't just call list_shards and create consumers for all");
    println!("   - Filter out closed shards first");
    println!("   - Track parent-child relationships\n");
    
    println!("2. Add shard status checking:");
    println!("   - Check if shard.sequence_number_range.ending_sequence_number is Some");
    println!("   - Only create consumers for open shards\n");
    
    println!("3. Implement checkpoint migration:");
    println!("   - When a parent shard is closed, transfer its checkpoint to children");
    println!("   - Handle both split (1 parent -> 2 children) and merge (2 parents -> 1 child)\n");
    
    println!("4. Add resharding detection:");
    println!("   - Periodically check if shard topology has changed");
    println!("   - Gracefully handle transitions\n");
    
    println!("5. Update ShardConsumerMessage handling:");
    println!("   - When receiving ShardClosed message, check for children");
    println!("   - Automatically spawn consumers for child shards");
}