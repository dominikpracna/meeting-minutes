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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-6);

        let v3 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&v1, &v3)).abs() < 1e-6);
    }

    #[test]
    fn test_clustering_new_speaker() {
        let mut clustering = OnlineClustering::new(0.5);
        let emb1 = vec![1.0, 0.0];
        let id1 = clustering.process_segment(&emb1);
        assert_eq!(id1, 1);

        let emb2 = vec![0.0, 1.0]; // Orthogonal, should be new speaker
        let id2 = clustering.process_segment(&emb2);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_clustering_same_speaker() {
        let mut clustering = OnlineClustering::new(0.5);
        let emb1 = vec![1.0, 0.0];
        let id1 = clustering.process_segment(&emb1);

        let emb2 = vec![0.9, 0.1]; // Close enough
        let id2 = clustering.process_segment(&emb2);
        assert_eq!(id1, id2);
    }
}
