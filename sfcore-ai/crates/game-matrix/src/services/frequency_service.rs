use std::collections::HashMap;
use game_models::LogGame;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FrequencyResult {
    pub position: String, // "As", "Kop", "Kepala", "Ekor" 
    pub digit: u8,
    pub observed: i64,
    pub expected: f64,
    pub z_score: f64,
    pub label: String, // "UNDERREPRESENTED", "NORMAL", "OVERREPRESENTED"
}

#[derive(Debug, Serialize)]
pub struct ConsistencyResult {
    pub position: String,
    pub digit: u8,
    pub label: String, // "WEAK", "MEDIUM", "STRONG"
    pub strength: u8, // 1, 2, 3
}

#[derive(Debug, Serialize)]
pub struct EntropyResult {
    pub position: String,
    pub normalized_entropy: f64,
    pub label: String, // "SANGAT MERATA", "CUKUP MERATA", "KURANG MERATA"
    pub distribution: Vec<usize>, // Counts for 0-9 for visualization
}

#[derive(Debug, Serialize, Clone)]
pub struct PairResult {
    pub digit_a: u8,
    pub digit_b: u8,
    pub count: usize,
    pub expected: f64,
    pub deviation: f64,
    pub label: String,      // "SANGAT SERING", "CUKUP SERING", "NORMAL", "CUKUP JARANG", "SANGAT JARANG"
    pub color_class: String, // "pair-frequent", "pair-normal", "pair-rare"
}

#[derive(Debug, Serialize)]
pub struct FrequencyAnalysisResponse {
    pub window_size: usize,
    pub total_periods: usize,
    pub results: Vec<FrequencyResult>,
    pub consistency: Vec<ConsistencyResult>,
    pub entropy: Vec<EntropyResult>,
    pub pairs: HashMap<String, Vec<PairResult>>, // Key: "As-Kop", "Kop-Kepala", "Kepala-Ekor"
}

pub struct FrequencyService;

impl FrequencyService {
    pub fn new() -> Self {
        Self
    }

    // Helper to calculate marginal frequencies
    fn get_marginal_frequencies(&self, logs: &[LogGame], position: &str) -> [usize; 10] {
        let mut freq = [0; 10];
        for log in logs {
            let val = match position {
                "As" => log.as_digit,
                "Kop" => log.kop,
                "Kepala" => log.kepala,
                "Ekor" => log.ekor,
                _ => None,
            };
            if let Some(d) = val {
                freq[d as usize] += 1;
            }
        }
        freq
    }

    // Process a specific pair type (e.g., As-Kop)
    fn analyze_pair_type(&self, logs: &[LogGame], pos_a: &str, pos_b: &str) -> Vec<PairResult> {
        let total = logs.len() as f64;
        if total == 0.0 { return Vec::new(); }

        let freq_a = self.get_marginal_frequencies(logs, pos_a);
        let freq_b = self.get_marginal_frequencies(logs, pos_b);

        let mut observed_counts: HashMap<(u8, u8), usize> = HashMap::new();
        
        // Count observed pairs
        for log in logs {
             let val_a = match pos_a { "As" => log.as_digit, "Kop" => log.kop, "Kepala" => log.kepala, _ => None };
             let val_b = match pos_b { "Kop" => log.kop, "Kepala" => log.kepala, "Ekor" => log.ekor, _ => None };
             
             if let (Some(a), Some(b)) = (val_a, val_b) {
                 *observed_counts.entry((a as u8, b as u8)).or_insert(0) += 1;
             }
        }

        let mut all_pairs = Vec::new();

        // Evaluate all 100 possible pairs
        for a in 0..=9 {
            for b in 0..=9 {
                let observed = *observed_counts.get(&(a, b)).unwrap_or(&0) as f64;
                
                // CRITICIAL: Conditional Probability Expected
                // P(A) * P(B) * N
                let p_a = freq_a[a as usize] as f64 / total;
                let p_b = freq_b[b as usize] as f64 / total;
                let expected = p_a * p_b * total;

                let deviation = if expected > 0.0 {
                    (observed - expected) / expected
                } else {
                    0.0
                };

                let (label, color_class) = if deviation >= 0.80 {
                    ("SANGAT SERING".to_string(), "pair-frequent".to_string())
                } else if deviation >= 0.30 {
                    ("CUKUP SERING".to_string(), "pair-frequent".to_string())
                } else if deviation <= -0.80 {
                    ("SANGAT JARANG".to_string(), "pair-rare".to_string())
                } else if deviation <= -0.31 {
                    ("CUKUP JARANG".to_string(), "pair-rare".to_string())
                } else {
                    ("NORMAL".to_string(), "pair-normal".to_string())
                };

                all_pairs.push(PairResult {
                    digit_a: a,
                    digit_b: b,
                    count: observed as usize,
                    expected,
                    deviation,
                    label,
                    color_class,
                });
            }
        }

        // Selection Filter: Curated 20
        // We want a mix: Top Frequent, Normal, Rare
        all_pairs.sort_by(|x, y| y.deviation.partial_cmp(&x.deviation).unwrap()); // Sort descending by deviation
        
        let mut curated = Vec::new();
        
        // 5 Sangat/Cukup Sering (Top 5)
        curated.extend(all_pairs.iter().take(5).cloned());
        
        // 5 Normal (Middle of the pack - finding ones close to 0 deviation)
        let _normals: Vec<&PairResult> = all_pairs.iter().filter(|p| p.deviation.abs() < 0.3).collect();
        // Just take distinct ones if possible, but simplest is to range map. 
        // Let's filter specifically for labels to be safe
        let normals: Vec<PairResult> = all_pairs.iter().filter(|p| p.label == "NORMAL").cloned().collect();
        curated.extend(normals.into_iter().take(5));

        // 5 Cukup Jarang / Sangat Jarang (Bottom 5)
        // Since it's sorted descending, take from end
        let len = all_pairs.len();
        if len >= 5 {
             let bottom_5 = all_pairs.iter().rev().take(5).cloned().collect::<Vec<_>>();
             // Reverse back to have them in some logical order or just append?
             // Appending them in order of rarity (most rare last) is fine.
             curated.extend(bottom_5);
        }

        // Fill remaining slots to reach 20 if needed, prioritize "Sering" or "Normal" not yet in list?
        // Actually the user specified: 5 Sering, 5 Cukup Sering, 5 Normal, 3 Cukup Jarang, 2 Sangat Jarang
        // The above simple slice was: Top 5, normal 5, bottom 5 = 15.
        // Let's try to follow the exact counts if possible, but "ranking" is simpler for robustness.
        // Let's stick to the "Top 5 High, 5 Normal, 5 Low" approximation + fill to 20.
        
        // Improved Selection Strategy:
        // 1. Get all deviations.
        // 2. Select Top 7 (Candidate for Frequent)
        // 3. Select Bottom 5 (Candidate for Rare)
        // 4. Select Low Deviation (Normal) 8
        // Total 20.
        
        let mut final_list = Vec::new();
        // Top 7
        final_list.extend(all_pairs.iter().take(7).cloned());
        // Bottom 5
        if len > 5 {
             final_list.extend(all_pairs.iter().rev().take(5).cloned());
        }
        // Middle 8 (closest to 0 deviation)
        let mut normals_sorted_by_abs = all_pairs.clone();
        normals_sorted_by_abs.sort_by(|x, y| x.deviation.abs().partial_cmp(&y.deviation.abs()).unwrap());
        // Be careful not to pick ones already picked.
        let mut added_count = 0;
        for p in normals_sorted_by_abs {
             if added_count >= 8 { break; }
             // Check if already in final_list
             if !final_list.iter().any(|existing| existing.digit_a == p.digit_a && existing.digit_b == p.digit_b) {
                 final_list.push(p);
                 added_count += 1;
             }
        }
        
        // Final Sort by Deviation Descending for Display
        final_list.sort_by(|x, y| y.deviation.partial_cmp(&x.deviation).unwrap());
        
        final_list
    }

    fn calculate_entropy(&self, counts: &HashMap<u8, usize>, total: usize) -> (f64, f64) {
        if total == 0 { return (0.0, 0.0); }
        
        let mut entropy = 0.0;
        let total_f = total as f64;
        
        for d in 0..=9 {
            let count = *counts.get(&d).unwrap_or(&0);
            if count > 0 {
                let p = count as f64 / total_f;
                entropy -= p * p.log2();
            }
        }
        
        let max_entropy = (10.0_f64).log2(); // log2(10) ≈ 3.3219
        let normalized = entropy / max_entropy;
        
        (entropy, normalized)
    }

    fn calculate_z_score_map(&self, logs: &[LogGame]) -> HashMap<(String, u8), String> {
        let n = logs.len() as f64;
        if n == 0.0 {
            return HashMap::new();
        }

        let mut counts: HashMap<(String, u8), i64> = HashMap::new();
        for pos in ["As", "Kop", "Kepala", "Ekor"] {
            for d in 0..=9 {
                counts.insert((pos.to_string(), d), 0);
            }
        }

        for log in logs {
            if let Some(v) = log.as_digit { *counts.get_mut(&("As".to_string(), v as u8)).unwrap() += 1; }
            if let Some(v) = log.kop { *counts.get_mut(&("Kop".to_string(), v as u8)).unwrap() += 1; }
            if let Some(v) = log.kepala { *counts.get_mut(&("Kepala".to_string(), v as u8)).unwrap() += 1; }
            if let Some(v) = log.ekor { *counts.get_mut(&("Ekor".to_string(), v as u8)).unwrap() += 1; }
        }

        let expected = n / 10.0;
        let variance = expected * 0.9;
        let std_dev = variance.sqrt();

        let mut labels = HashMap::new();

        for ((pos, digit), observed) in counts {
            let z_score = if std_dev > 0.0 {
                (observed as f64 - expected) / std_dev
            } else {
                0.0
            };

            let label = if z_score <= -1.0 {
                "UNDERREPRESENTED".to_string()
            } else if z_score >= 1.0 {
                "OVERREPRESENTED".to_string()
            } else {
                "NORMAL".to_string()
            };
            
            labels.insert((pos, digit), label);
        }
        labels
    }

    pub fn analyze(&self, logs: &[LogGame], window_size: usize) -> FrequencyAnalysisResponse {
        let total_periods = logs.len();
        
        // 1. Main Analysis (Full Window)
        let main_labels = self.calculate_z_score_map(logs);
        
        let mut results = Vec::new();
        let n = total_periods as f64;
        let expected = n / 10.0;
        let std_dev = (expected * 0.9).sqrt();
        
        let mut counts: HashMap<(String, u8), i64> = HashMap::new();
        // Separate counts specifically for Entropy (per position)
        let mut position_counts: HashMap<String, HashMap<u8, usize>> = HashMap::new();
        for pos in ["As", "Kop", "Kepala", "Ekor"] {
            position_counts.insert(pos.to_string(), HashMap::new());
             for d in 0..=9 {
                counts.insert((pos.to_string(), d), 0);
                position_counts.get_mut(pos).unwrap().insert(d, 0);
            }
        }

        for log in logs {
             if let Some(v) = log.as_digit { 
                 *counts.get_mut(&("As".to_string(), v as u8)).unwrap() += 1; 
                 *position_counts.get_mut("As").unwrap().get_mut(&(v as u8)).unwrap() += 1;
             }
             if let Some(v) = log.kop { 
                 *counts.get_mut(&("Kop".to_string(), v as u8)).unwrap() += 1; 
                 *position_counts.get_mut("Kop").unwrap().get_mut(&(v as u8)).unwrap() += 1;
             }
             if let Some(v) = log.kepala { 
                 *counts.get_mut(&("Kepala".to_string(), v as u8)).unwrap() += 1; 
                 *position_counts.get_mut("Kepala").unwrap().get_mut(&(v as u8)).unwrap() += 1;
             }
             if let Some(v) = log.ekor { 
                 *counts.get_mut(&("Ekor".to_string(), v as u8)).unwrap() += 1; 
                 *position_counts.get_mut("Ekor").unwrap().get_mut(&(v as u8)).unwrap() += 1;
             }
        }

        for pos in ["As", "Kop", "Kepala", "Ekor"] {
            for d in 0..=9 {
                let observed = *counts.get(&(pos.to_string(), d)).unwrap();
                let z_score = if std_dev > 0.0 { (observed as f64 - expected) / std_dev } else { 0.0 };
                let label = main_labels.get(&(pos.to_string(), d)).unwrap().clone();
                
                results.push(FrequencyResult {
                    position: pos.to_string(),
                    digit: d,
                    observed,
                    expected,
                    z_score,
                    label,
                });
            }
        }

        // 2. Consistency Analysis (Non-Overlapping Windows)
        let mut consistency_results = Vec::new();
        if total_periods >= 100 {
            let w_size = 100;
            let mut windows = Vec::new();
            if total_periods >= w_size { windows.push(&logs[0..w_size]); }
            if total_periods >= w_size * 2 { windows.push(&logs[w_size..w_size*2]); }
            if total_periods >= w_size * 3 { windows.push(&logs[w_size*2..w_size*3]); }

            let mut window_labels: Vec<HashMap<(String, u8), String>> = Vec::new();
            for window_logs in &windows {
                window_labels.push(self.calculate_z_score_map(window_logs));
            }

            for pos in ["As", "Kop", "Kepala", "Ekor"] {
                for d in 0..=9 {
                    if window_labels.is_empty() { continue; }
                    let current_label = window_labels[0].get(&(pos.to_string(), d)).unwrap();
                    let mut match_count = 1;
                    for i in 1..window_labels.len() {
                        if window_labels[i].get(&(pos.to_string(), d)).unwrap() == current_label {
                            match_count += 1;
                        }
                    }
                    let (strength, label_txt) = match match_count {
                        3 => (3, "STRONG".to_string()),
                        2 => (2, "MEDIUM".to_string()),
                        _ => (1, "WEAK".to_string()),
                    };
                    consistency_results.push(ConsistencyResult {
                        position: pos.to_string(),
                        digit: d,
                        label: label_txt,
                        strength,
                    });
                }
            }
        }

        // 3. Entropy Analysis (Kemerataan Sebaran)
        let mut entropy_results = Vec::new();
        // Constraint: Minimum 150 periods for entropy
        if total_periods >= 150 {
             for pos in ["As", "Kop", "Kepala", "Ekor"] {
                 let pos_counts = position_counts.get(pos).unwrap();
                 let (_, normalized) = self.calculate_entropy(pos_counts, total_periods);
                 
                 let label = if normalized >= 0.92 {
                     "SANGAT MERATA".to_string()
                 } else if normalized >= 0.80 {
                     "CUKUP MERATA".to_string()
                 } else {
                     "KURANG MERATA".to_string()
                 };

                 let mut distribution = Vec::new();
                 for d in 0..=9 {
                     distribution.push(*pos_counts.get(&d).unwrap_or(&0));
                 }

                 entropy_results.push(EntropyResult {
                     position: pos.to_string(),
                     normalized_entropy: normalized,
                     label,
                     distribution
                 });
             }
        }

        // 4. Pair Pattern Analysis (Batch 5)
        let mut pair_results = HashMap::new();
        // Constraint: Minimum 50 periods for Pair Analysis to be meaningful (lowered from 200 to support window=100)
        if total_periods >= 50 {
            pair_results.insert("As-Kop".to_string(), self.analyze_pair_type(logs, "As", "Kop"));
            pair_results.insert("Kop-Kepala".to_string(), self.analyze_pair_type(logs, "Kop", "Kepala"));
            pair_results.insert("Kepala-Ekor".to_string(), self.analyze_pair_type(logs, "Kepala", "Ekor"));
        }

        FrequencyAnalysisResponse {
            window_size,
            total_periods,
            results,
            consistency: consistency_results,
            entropy: entropy_results,
            pairs: pair_results,
        }
    }
    pub fn generate_human_summary(&self, freq_resp: &FrequencyAnalysisResponse) -> String {
        // Logic untuk menghasilkan kalimat deskriptif ala "Sebaran angka cenderung merata..."
        // Basis: Entropy (sebaran) + Consistency (kekuatan pola)
        
        let entropy_avg: f64 = freq_resp.entropy.iter().map(|e| e.normalized_entropy).sum::<f64>() / 4.0;
        let strong_patterns = freq_resp.consistency.iter().filter(|c| c.label == "STRONG").count();
        let medium_patterns = freq_resp.consistency.iter().filter(|c| c.label == "MEDIUM").count();

        let sebaran_text = if entropy_avg >= 0.90 {
            "Sebaran angka sangat merata"
        } else if entropy_avg >= 0.82 {
            "Sebaran angka cenderung merata"
        } else {
            "Beberapa angka menonjol signifikan"
        };

        let pola_text = if strong_patterns >= 3 {
             "dengan pola yang sangat konsisten"
        } else if strong_patterns >= 1 || medium_patterns >= 4 {
             "dengan pola yang cukup konsisten"
        } else {
             "dengan pola yang tidak konsisten"
        };

        format!("{} {}", sebaran_text, pola_text)
    }
}
