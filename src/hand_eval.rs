use crate::card::CardInstance;
use crate::types::{HandType, Rank, Suit};
use std::collections::HashMap;

/// Result of hand evaluation: the best hand type and which card indices are scoring
#[derive(Debug, Clone)]
pub struct HandEvalResult {
    pub hand_type: HandType,
    /// Indices into the played cards slice that are part of the scoring hand
    pub scoring_indices: Vec<usize>,
}

/// Evaluate a set of played cards (up to 5) and return the best hand + scoring cards.
/// `four_fingers` enables flushes/straights with only 4 cards.
/// `shortcut` allows straights with gaps.
/// `smeared` treats Hearts=Diamonds and Spades=Clubs for flush purposes.
pub fn evaluate_hand(
    cards: &[CardInstance],
    four_fingers: bool,
    shortcut: bool,
    smeared: bool,
    splash: bool,
) -> HandEvalResult {
    // Collect non-stone cards for hand evaluation (stone cards don't count for hand type)
    let eval_indices: Vec<usize> = cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.enhancement != crate::types::Enhancement::Stone)
        .map(|(i, _)| i)
        .collect();

    let eval_cards: Vec<&CardInstance> = eval_indices.iter().map(|&i| &cards[i]).collect();

    let flush_threshold = if four_fingers { 4 } else { 5 };
    let straight_threshold = if four_fingers { 4 } else { 5 };

    // Check flush
    let flush_result = check_flush(&eval_cards, &eval_indices, flush_threshold, smeared);
    // Check straight
    let straight_result = check_straight(&eval_cards, &eval_indices, straight_threshold, shortcut);
    // Check groups (pairs, trips, quads, quints)
    let groups = get_rank_groups(&eval_cards, &eval_indices);

    // Now determine best hand
    // Flush Five: 5+ cards of same suit + same rank
    if let Some(ref flush_idxs) = flush_result {
        if flush_idxs.len() >= 5 {
            let flush_cards: Vec<&CardInstance> = flush_idxs.iter().map(|&i| &cards[i]).collect();
            if let Some(five_kind) = find_five_of_kind(&flush_cards, &flush_idxs[..]) {
                if five_kind.len() >= 5 {
                    return HandEvalResult {
                        hand_type: HandType::FlushFive,
                        scoring_indices: five_kind,
                    };
                }
            }
            // Flush House: full house all same suit
            if let Some(fh) = find_full_house(&flush_cards, &flush_idxs[..]) {
                if fh.len() >= 5 {
                    return HandEvalResult {
                        hand_type: HandType::FlushHouse,
                        scoring_indices: fh,
                    };
                }
            }
        }
    }

    // Five of a Kind
    if let Some(five) = find_five_of_kind(&eval_cards, &eval_indices) {
        if five.len() >= 5 {
            return HandEvalResult {
                hand_type: HandType::FiveOfAKind,
                scoring_indices: five,
            };
        }
    }

    // Straight Flush
    if let (Some(flush_idxs), Some(straight_idxs)) = (&flush_result, &straight_result) {
        // Intersection: cards that are in both flush and straight
        let sf_idxs: Vec<usize> = straight_idxs
            .iter()
            .filter(|i| flush_idxs.contains(i))
            .copied()
            .collect();
        if sf_idxs.len() >= straight_threshold {
            return HandEvalResult {
                hand_type: HandType::StraightFlush,
                scoring_indices: sf_idxs,
            };
        }
    }

    // Four of a Kind
    if let Some((four_idxs, _kicker)) = find_four_of_kind(&groups, &eval_indices) {
        let scoring = if splash {
            eval_indices.clone()
        } else {
            four_idxs
        };
        return HandEvalResult {
            hand_type: HandType::FourOfAKind,
            scoring_indices: scoring,
        };
    }

    // Full House
    if let Some(fh_idxs) = find_full_house(&eval_cards, &eval_indices) {
        return HandEvalResult {
            hand_type: HandType::FullHouse,
            scoring_indices: fh_idxs,
        };
    }

    // Flush
    if let Some(flush_idxs) = flush_result {
        if flush_idxs.len() >= flush_threshold {
            return HandEvalResult {
                hand_type: HandType::Flush,
                scoring_indices: flush_idxs,
            };
        }
    }

    // Straight
    if let Some(straight_idxs) = straight_result {
        if straight_idxs.len() >= straight_threshold {
            return HandEvalResult {
                hand_type: HandType::Straight,
                scoring_indices: straight_idxs,
            };
        }
    }

    // Three of a Kind
    if let Some((trip_idxs, _)) = find_three_of_kind(&groups, &eval_indices) {
        let scoring = if splash {
            eval_indices.clone()
        } else {
            trip_idxs
        };
        return HandEvalResult {
            hand_type: HandType::ThreeOfAKind,
            scoring_indices: scoring,
        };
    }

    // Two Pair
    if let Some(tp_idxs) = find_two_pair(&groups, &eval_indices) {
        let scoring = if splash {
            eval_indices.clone()
        } else {
            tp_idxs
        };
        return HandEvalResult {
            hand_type: HandType::TwoPair,
            scoring_indices: scoring,
        };
    }

    // Pair
    if let Some((pair_idxs, _)) = find_pair(&groups, &eval_indices) {
        let scoring = if splash {
            eval_indices.clone()
        } else {
            pair_idxs
        };
        return HandEvalResult {
            hand_type: HandType::Pair,
            scoring_indices: scoring,
        };
    }

    // High Card - highest single card
    let best_idx = eval_indices
        .iter()
        .max_by_key(|&&i| eval_cards[eval_indices.iter().position(|&x| x == i).unwrap()].rank.numeric_value())
        .copied()
        .unwrap_or(0);
    HandEvalResult {
        hand_type: HandType::HighCard,
        scoring_indices: if splash {
            eval_indices
        } else {
            vec![best_idx]
        },
    }
}

fn check_flush(
    cards: &[&CardInstance],
    indices: &[usize],
    threshold: usize,
    smeared: bool,
) -> Option<Vec<usize>> {
    if cards.is_empty() {
        return None;
    }

    // Count suits (handling wild cards)
    // For flush: need enough cards of same suit (or wild)
    let suit_fn = |s: Suit| -> Suit {
        if smeared {
            match s {
                Suit::Hearts | Suit::Diamonds => Suit::Hearts,
                Suit::Spades | Suit::Clubs => Suit::Spades,
            }
        } else {
            s
        }
    };

    // Group cards by effective suit
    let mut suit_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut wild_indices: Vec<usize> = Vec::new();

    for (pos, (&card, &orig_idx)) in cards.iter().zip(indices.iter()).enumerate() {
        if card.enhancement == crate::types::Enhancement::Wild {
            wild_indices.push(orig_idx);
        } else {
            let key = format!("{:?}", suit_fn(card.suit));
            suit_groups.entry(key).or_default().push(orig_idx);
        }
    }

    // Find the suit with the most cards
    let best_suit_idxs: Option<Vec<usize>> = suit_groups
        .into_values()
        .max_by_key(|v| v.len())
        .filter(|v| v.len() + wild_indices.len() >= threshold);

    best_suit_idxs.map(|mut v| {
        v.extend(wild_indices.iter().copied());
        v
    })
}

fn check_straight(
    cards: &[&CardInstance],
    indices: &[usize],
    threshold: usize,
    shortcut: bool,
) -> Option<Vec<usize>> {
    if cards.len() < threshold {
        return None;
    }

    // Get unique ranks (excluding stone cards, which have no rank for this purpose)
    // Pair each rank with its original index
    let mut rank_idx: Vec<(u8, usize)> = cards
        .iter()
        .zip(indices.iter())
        .filter(|(c, _)| c.enhancement != crate::types::Enhancement::Stone)
        .map(|(c, &i)| (c.rank.numeric_value(), i))
        .collect();

    // Deduplicate by rank (take lowest index for each unique rank)
    rank_idx.sort_by_key(|&(r, _)| r);
    let mut unique: Vec<(u8, usize)> = Vec::new();
    for item in &rank_idx {
        if unique.last().map(|(r, _)| *r) != Some(item.0) {
            unique.push(*item);
        }
    }

    // Try with Ace = 1 as well
    let mut results = Vec::new();
    for use_ace_low in [false, true] {
        let mut vals: Vec<(u8, usize)> = unique.clone();
        if use_ace_low {
            // Replace Ace (14) with 1
            for (r, i) in vals.iter_mut() {
                if *r == 14 {
                    *r = 1;
                }
            }
            vals.sort_by_key(|&(r, _)| r);
        }

        // Find longest consecutive run
        let best = find_consecutive_run(&vals, threshold, shortcut);
        if let Some(run) = best {
            results.push(run);
        }
    }

    results.into_iter().max_by_key(|v| v.len())
}

fn find_consecutive_run(
    vals: &[(u8, usize)],
    threshold: usize,
    shortcut: bool,
) -> Option<Vec<usize>> {
    if vals.len() < threshold {
        return None;
    }

    // Try sliding window of 5 (or threshold) consecutive values
    // shortcut: allows one gap
    let n = vals.len();
    let mut best: Option<Vec<usize>> = None;

    for start in 0..n {
        let mut run = vec![vals[start].1];
        let mut last_val = vals[start].0;
        let mut gaps_used = 0;
        let max_gaps = if shortcut { 1 } else { 0 };

        for pos in (start + 1)..n {
            let diff = vals[pos].0 as i32 - last_val as i32;
            if diff == 1 {
                run.push(vals[pos].1);
                last_val = vals[pos].0;
            } else if diff == 2 && gaps_used < max_gaps {
                // Allow one gap for shortcut
                run.push(vals[pos].1);
                last_val = vals[pos].0;
                gaps_used += 1;
            } else if diff > 2 || (diff == 2 && gaps_used >= max_gaps) {
                break;
            }
            // diff == 0 means duplicate rank, skip
        }

        if run.len() >= threshold {
            if best.as_ref().map(|b| b.len()).unwrap_or(0) < run.len() {
                best = Some(run);
            }
        }
    }

    best
}

/// Returns rank → list of card indices
fn get_rank_groups(
    cards: &[&CardInstance],
    indices: &[usize],
) -> HashMap<Rank, Vec<usize>> {
    let mut groups: HashMap<Rank, Vec<usize>> = HashMap::new();
    for (&card, &idx) in cards.iter().zip(indices.iter()) {
        if card.enhancement != crate::types::Enhancement::Stone {
            groups.entry(card.rank).or_default().push(idx);
        }
    }
    groups
}

fn find_five_of_kind(cards: &[&CardInstance], indices: &[usize]) -> Option<Vec<usize>> {
    let groups = get_rank_groups(cards, indices);
    groups
        .values()
        .find(|v| v.len() >= 5)
        .cloned()
}

fn find_four_of_kind(
    groups: &HashMap<Rank, Vec<usize>>,
    _all_indices: &[usize],
) -> Option<(Vec<usize>, Vec<usize>)> {
    let four = groups.values().find(|v| v.len() >= 4)?;
    let kicker: Vec<usize> = groups
        .values()
        .filter(|v| v.len() < 4)
        .flat_map(|v| v.iter().copied())
        .collect();
    Some((four.clone(), kicker))
}

fn find_three_of_kind(
    groups: &HashMap<Rank, Vec<usize>>,
    _all_indices: &[usize],
) -> Option<(Vec<usize>, Vec<usize>)> {
    let trip = groups.values().find(|v| v.len() >= 3)?;
    let kicker: Vec<usize> = groups
        .values()
        .filter(|v| v.as_ptr() != trip.as_ptr() && v.len() < 3)
        .flat_map(|v| v.iter().copied())
        .collect();
    Some((trip.clone(), kicker))
}

fn find_full_house(cards: &[&CardInstance], indices: &[usize]) -> Option<Vec<usize>> {
    let groups = get_rank_groups(cards, indices);
    let mut trips: Vec<&Vec<usize>> = groups.values().filter(|v| v.len() >= 3).collect();
    let pairs: Vec<&Vec<usize>> = groups.values().filter(|v| v.len() == 2).collect();

    // Need either trip + pair, or two trips (take 3 from one, 2 from other)
    if trips.len() >= 2 {
        trips.sort_by_key(|v| std::cmp::Reverse(v.len()));
        let mut result: Vec<usize> = trips[0].iter().take(3).copied().collect();
        result.extend(trips[1].iter().take(2).copied());
        return Some(result);
    }

    if trips.len() == 1 && !pairs.is_empty() {
        let mut result: Vec<usize> = trips[0].iter().take(3).copied().collect();
        result.extend(pairs[0].iter().copied());
        return Some(result);
    }

    None
}

fn find_two_pair(
    groups: &HashMap<Rank, Vec<usize>>,
    _all_indices: &[usize],
) -> Option<Vec<usize>> {
    let pairs: Vec<&Vec<usize>> = groups.values().filter(|v| v.len() >= 2).collect();
    if pairs.len() < 2 {
        return None;
    }
    // Take the two highest pairs
    let mut sorted_pairs: Vec<(&Rank, &Vec<usize>)> = groups
        .iter()
        .filter(|(_, v)| v.len() >= 2)
        .collect();
    sorted_pairs.sort_by_key(|(r, _)| std::cmp::Reverse(r.numeric_value()));

    let mut result: Vec<usize> = sorted_pairs[0].1.iter().take(2).copied().collect();
    result.extend(sorted_pairs[1].1.iter().take(2).copied());
    Some(result)
}

fn find_pair(
    groups: &HashMap<Rank, Vec<usize>>,
    _all_indices: &[usize],
) -> Option<(Vec<usize>, Vec<usize>)> {
    let mut pairs: Vec<(&Rank, &Vec<usize>)> = groups
        .iter()
        .filter(|(_, v)| v.len() >= 2)
        .collect();
    if pairs.is_empty() {
        return None;
    }
    pairs.sort_by_key(|(r, _)| std::cmp::Reverse(r.numeric_value()));
    let pair = pairs[0].1.iter().take(2).copied().collect::<Vec<_>>();
    let kicker: Vec<usize> = groups
        .values()
        .filter(|v| {
            !v.iter().any(|i| pair.contains(i))
        })
        .flat_map(|v| v.iter().copied())
        .collect();
    Some((pair, kicker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardInstance;
    use crate::types::{Enhancement, Rank, Suit};

    fn make_card(id: u64, rank: Rank, suit: Suit) -> CardInstance {
        CardInstance::new(id, rank, suit)
    }

    #[test]
    fn test_pair() {
        let cards = vec![
            make_card(0, Rank::Ace, Suit::Spades),
            make_card(1, Rank::Ace, Suit::Hearts),
            make_card(2, Rank::Three, Suit::Clubs),
            make_card(3, Rank::Seven, Suit::Diamonds),
            make_card(4, Rank::Nine, Suit::Spades),
        ];
        let result = evaluate_hand(&cards, false, false, false, false);
        assert_eq!(result.hand_type, HandType::Pair);
        assert_eq!(result.scoring_indices.len(), 2);
    }

    #[test]
    fn test_flush() {
        let cards = vec![
            make_card(0, Rank::Ace, Suit::Spades),
            make_card(1, Rank::Three, Suit::Spades),
            make_card(2, Rank::Seven, Suit::Spades),
            make_card(3, Rank::Nine, Suit::Spades),
            make_card(4, Rank::Two, Suit::Spades),
        ];
        let result = evaluate_hand(&cards, false, false, false, false);
        assert_eq!(result.hand_type, HandType::Flush);
    }

    #[test]
    fn test_straight() {
        let cards = vec![
            make_card(0, Rank::Five, Suit::Spades),
            make_card(1, Rank::Six, Suit::Hearts),
            make_card(2, Rank::Seven, Suit::Clubs),
            make_card(3, Rank::Eight, Suit::Diamonds),
            make_card(4, Rank::Nine, Suit::Spades),
        ];
        let result = evaluate_hand(&cards, false, false, false, false);
        assert_eq!(result.hand_type, HandType::Straight);
    }
}
