use std::collections::HashMap;

/// Simple Hierarchical Agglomerative Clustering
pub struct OnlineClustering {
    clusters: HashMap<usize, Vec<f32>>, // Cluster ID -> Centroid
    threshold: f32,
    next_id: usize,
}

impl OnlineClustering {
    pub fn new(threshold: f32) -> Self {
        Self {
            clusters: HashMap::new(),
            threshold,
            next_id: 1, // Start Speaker IDs from 1
        }
    }

    pub fn process_segment(&mut self, embedding: &[f32]) -> usize {
        let mut best_cluster = None;
        let mut max_sim = -1.0;

        // Find best matching cluster
        for (id, centroid) in &self.clusters {
            let sim = cosine_similarity(embedding, centroid);
            if sim > max_sim {
                max_sim = sim;
                best_cluster = Some(*id);
            }
        }

        if let Some(id) = best_cluster {
            if max_sim >= self.threshold {
                // Update centroid (simple moving average for online)
                // Or just keep the centroid if it's representative.
                // For simplicity, let's just return the ID.
                // A better update would be: new_centroid = (old_centroid * N + new_emb) / (N + 1)
                // But we don't track N here. Let's do a simple weighted update.
                let centroid = self.clusters.get_mut(&id).unwrap();
                for i in 0..centroid.len() {
                    centroid[i] = 0.9 * centroid[i] + 0.1 * embedding[i];
                }
                return id;
            }
        }

        // New speaker
        let id = self.next_id;
        self.next_id += 1;
        self.clusters.insert(id, embedding.to_vec());
        id
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
