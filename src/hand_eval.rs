use crate::card::CardInstance;
use crate::types::{HandType, Rank, Suit};
use std::collections::HashMap;

/// Every poker hand the played cards contain, not just the best one.
///
/// Balatro builds this table in `evaluate_poker_hand` (misc_functions.lua:376) and the
/// hand-shape jokers test it rather than the hand's name: Jolly Joker asks
/// `next(context.poker_hands['Pair'])`, so a Flush that happens to hold a pair pays out.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContainedHands(u16);

impl ContainedHands {
    fn bit(hand: HandType) -> u16 {
        1u16 << (hand as u16)
    }

    pub fn insert(&mut self, hand: HandType) {
        self.0 |= Self::bit(hand);
    }

    pub fn contains(&self, hand: HandType) -> bool {
        self.0 & Self::bit(hand) != 0
    }
}

/// Result of hand evaluation: the best hand type and which card indices are scoring
#[derive(Debug, Clone)]
pub struct HandEvalResult {
    pub hand_type: HandType,
    /// Indices into the played cards slice that are part of the scoring hand
    pub scoring_indices: Vec<usize>,
    /// Every hand the played cards contain — what the hand-shape jokers key off.
    pub contained: ContainedHands,
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
    let parts = hand_parts(cards, four_fingers, shortcut, smeared);
    let mut result = best_hand(&parts);
    result.contained = contained_hands(&parts, !cards.is_empty());

    if splash {
        // Splash makes every played card score, whatever the hand is — Balatro overwrites the
        // whole scoring hand with `G.play.cards` (state_events.lua:583) rather than adding the
        // leftovers onto particular hand types.
        result.scoring_indices = (0..cards.len()).collect();
        return result;
    }

    // Stone cards take no part in deciding the hand type, but they always score. Balatro appends
    // them to the scoring hand as "pures" after the hand has been determined
    // (state_events.lua:581-599) — that is the whole point of the enhancement.
    for (i, c) in cards.iter().enumerate() {
        if c.is_stone() && !result.scoring_indices.contains(&i) {
            result.scoring_indices.push(i);
        }
    }

    // Balatro sorts the scoring hand left-to-right by on-screen position before scoring
    // (state_events.lua:600). Jokers that key off "the first scoring card" — Hanging Chad,
    // Photograph — depend on this, and grouping the hand by rank leaves them in an order that
    // depends on HashMap iteration, which is not stable between runs.
    result.scoring_indices.sort_unstable();
    result
}

/// The six building blocks `evaluate_poker_hand` works from (misc_functions.lua:394-401).
///
/// Both answers the caller wants — the hand that gets *played*, and the set of hands the cards
/// merely *contain* — are read off these same values, exactly as Balatro reads them off `parts`.
/// Deriving the two separately is how they drift apart, and the hand-shape jokers care about the
/// difference.
struct HandParts {
    /// Rank groups holding **exactly** five / four / three / two cards, best rank first.
    ///
    /// Exactness is load-bearing (`get_X_same` tests `#curr == num`): five of a kind leaves
    /// `four`, `three` and `two` empty, and it is the cascade at the end of `evaluate_poker_hand`
    /// that grants the smaller hands as *contained* rather than these groups.
    five: Vec<Vec<usize>>,
    four: Vec<Vec<usize>>,
    three: Vec<Vec<usize>>,
    two: Vec<Vec<usize>>,
    flush: Option<Vec<usize>>,
    straight: Option<Vec<usize>>,
    /// Highest-ranked card, which is what a High Card hand scores.
    highest: Option<usize>,
}

fn hand_parts(
    cards: &[CardInstance],
    four_fingers: bool,
    shortcut: bool,
    smeared: bool,
) -> HandParts {
    // Stone cards take no part in deciding anything: `Card:get_id` hands them a fresh random
    // negative on every call (card.lua:957), so they can never join a rank group or a straight,
    // and `is_suit` is false for them, so they can never join a flush. They are appended to the
    // scoring hand afterwards as "pures" instead.
    let eval_indices: Vec<usize> = cards
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_stone())
        .map(|(i, _)| i)
        .collect();

    // Four Fingers lets a flush or a straight get there with four cards instead of five.
    let threshold = if four_fingers { 4 } else { 5 };

    // `get_highest` keeps the first card of the winning rank — it compares with a strict `>`.
    let mut highest: Option<usize> = None;
    for &i in &eval_indices {
        let better = match highest {
            Some(h) => cards[i].rank.numeric_value() > cards[h].rank.numeric_value(),
            None => true,
        };
        if better {
            highest = Some(i);
        }
    }

    HandParts {
        five: same_rank_groups(cards, &eval_indices, 5),
        four: same_rank_groups(cards, &eval_indices, 4),
        three: same_rank_groups(cards, &eval_indices, 3),
        two: same_rank_groups(cards, &eval_indices, 2),
        flush: get_flush(cards, &eval_indices, threshold, smeared),
        straight: get_straight(cards, &eval_indices, threshold, shortcut),
        highest,
    }
}

/// `get_X_same` (misc_functions.lua:589): the rank groups holding **exactly** `n` cards, best
/// rank first.
fn same_rank_groups(
    cards: &[CardInstance],
    eval_indices: &[usize],
    n: usize,
) -> Vec<Vec<usize>> {
    let mut by_rank: HashMap<Rank, Vec<usize>> = HashMap::new();
    for &i in eval_indices {
        by_rank.entry(cards[i].rank).or_default().push(i);
    }
    let mut groups: Vec<(Rank, Vec<usize>)> =
        by_rank.into_iter().filter(|(_, v)| v.len() == n).collect();
    groups.sort_unstable_by_key(|(rank, _)| std::cmp::Reverse(rank.numeric_value()));
    groups.into_iter().map(|(_, v)| v).collect()
}

/// `get_flush` (misc_functions.lua:522).
///
/// The suits are tried in a fixed order and the **first** one to reach the threshold wins, rather
/// than the largest. That only shows when Wild Cards are in play — they answer yes to every suit,
/// so several suits can qualify at once and the tie has to break the same way Balatro breaks it.
fn get_flush(
    cards: &[CardInstance],
    eval_indices: &[usize],
    threshold: usize,
    smeared: bool,
) -> Option<Vec<usize>> {
    Suit::ALL.into_iter().find_map(|suit| {
        let group: Vec<usize> = eval_indices
            .iter()
            .copied()
            .filter(|&i| cards[i].is_suit(suit, smeared))
            .collect();
        (group.len() >= threshold).then_some(group)
    })
}

/// `get_straight` (misc_functions.lua:547), ported step for step.
///
/// Two details are easy to lose. The walk runs A,2,3,…,K,A, which is what lets an Ace close a
/// straight at either end without a separate ace-low pass. And Shortcut clears `skipped_rank`
/// every time it *does* find a rank, so a hand may bridge **several** single-rank gaps — 2,4,6,8,10
/// is a Straight — as long as no two gaps sit next to each other.
///
/// Every card of a matched rank joins the run, so a paired straight (5,5,6,7,8 under Four Fingers)
/// scores all five cards.
fn get_straight(
    cards: &[CardInstance],
    eval_indices: &[usize],
    threshold: usize,
    shortcut: bool,
) -> Option<Vec<usize>> {
    let mut by_id: HashMap<u8, Vec<usize>> = HashMap::new();
    for &i in eval_indices {
        by_id.entry(cards[i].rank.numeric_value()).or_default().push(i);
    }

    let mut run: Vec<usize> = Vec::new();
    let mut length = 0usize;
    let mut straight = false;
    let mut skipped = false;

    for step in 1..=14u8 {
        // Step 1 is the low Ace, step 14 the high one.
        let id = if step == 1 { 14 } else { step };
        match by_id.get(&id) {
            Some(group) => {
                length += 1;
                skipped = false;
                run.extend(group.iter().copied());
            }
            // A gap Shortcut can bridge. Not at the very last step, where there is nothing left
            // to bridge to.
            None if shortcut && !skipped && step != 14 => skipped = true,
            None => {
                length = 0;
                skipped = false;
                if straight {
                    // The straight is already settled; anything past it is not part of it.
                    break;
                }
                run.clear();
            }
        }
        if length >= threshold {
            straight = true;
        }
    }

    straight.then_some(run)
}

/// The hand that actually gets played, in `evaluate_poker_hand`'s order (misc_functions.lua:404).
/// The first rule to fire wins, so this sequence *is* the paytable, top to bottom.
fn best_hand(parts: &HandParts) -> HandEvalResult {
    let joined = |a: &[usize], b: &[usize]| -> Vec<usize> {
        a.iter().chain(b.iter()).copied().collect()
    };

    // The flush-and-something hands ask only that *a* flush exists — they do not re-derive the
    // rank groups from the flush's own cards. Under Four Fingers that lets a four-card flush plus
    // a full house score as a Flush House.
    if parts.flush.is_some() {
        if let Some(five) = parts.five.first() {
            return hand(HandType::FlushFive, five.clone());
        }
        if let (Some(three), Some(two)) = (parts.three.first(), parts.two.first()) {
            return hand(HandType::FlushHouse, joined(three, two));
        }
    }

    if let Some(five) = parts.five.first() {
        return hand(HandType::FiveOfAKind, five.clone());
    }

    // A Straight Flush scores the **union** of the two: every flush card, plus any straight card
    // not already among them (misc_functions.lua:427). There is no length test on the result —
    // with Four Fingers the two sets need not even overlap.
    if let (Some(flush), Some(straight)) = (parts.flush.as_ref(), parts.straight.as_ref()) {
        let mut both = flush.clone();
        both.extend(straight.iter().copied().filter(|i| !flush.contains(i)));
        return hand(HandType::StraightFlush, both);
    }

    if let Some(four) = parts.four.first() {
        return hand(HandType::FourOfAKind, four.clone());
    }
    if let (Some(three), Some(two)) = (parts.three.first(), parts.two.first()) {
        return hand(HandType::FullHouse, joined(three, two));
    }
    if let Some(flush) = parts.flush.as_ref() {
        return hand(HandType::Flush, flush.clone());
    }
    if let Some(straight) = parts.straight.as_ref() {
        return hand(HandType::Straight, straight.clone());
    }
    if let Some(three) = parts.three.first() {
        return hand(HandType::ThreeOfAKind, three.clone());
    }
    // Two separate exact pairs. The trip-plus-pair case that also grants Two Pair *containment*
    // cannot reach here — Full House claimed it above.
    if let [first, second, ..] = parts.two.as_slice() {
        return hand(HandType::TwoPair, joined(first, second));
    }
    if let Some(two) = parts.two.first() {
        return hand(HandType::Pair, two.clone());
    }

    hand(HandType::HighCard, parts.highest.into_iter().collect())
}

/// Every hand the played cards contain, following `evaluate_poker_hand` (misc_functions.lua:376).
///
/// Balatro's hand-shape jokers test this rather than the hand's name: Jolly Joker asks
/// `next(context.poker_hands['Pair'])`, so a Flush that happens to hold a pair pays out.
fn contained_hands(parts: &HandParts, any_cards: bool) -> ContainedHands {
    let mut out = ContainedHands::default();
    if !any_cards {
        return out;
    }

    let flush = parts.flush.is_some();
    let straight = parts.straight.is_some();
    let (n5, n4, n3, n2) = (
        parts.five.len(),
        parts.four.len(),
        parts.three.len(),
        parts.two.len(),
    );

    if n5 > 0 && flush { out.insert(HandType::FlushFive); }
    if n3 > 0 && n2 > 0 && flush { out.insert(HandType::FlushHouse); }
    if n5 > 0 { out.insert(HandType::FiveOfAKind); }
    if flush && straight { out.insert(HandType::StraightFlush); }
    if n4 > 0 { out.insert(HandType::FourOfAKind); }
    if n3 > 0 && n2 > 0 { out.insert(HandType::FullHouse); }
    if flush { out.insert(HandType::Flush); }
    if straight { out.insert(HandType::Straight); }
    if n3 > 0 { out.insert(HandType::ThreeOfAKind); }
    if n2 == 2 || (n3 == 1 && n2 == 1) { out.insert(HandType::TwoPair); }
    if n2 > 0 { out.insert(HandType::Pair); }
    // Any non-empty hand is at least a High Card, including one that is nothing but Stone cards.
    // Testing the evaluated cards here instead would leave an all-Stone hand containing no hand at
    // all, while `best_hand` still calls it a High Card — and the hand you played has to be among
    // the hands you hold.
    out.insert(HandType::HighCard);

    // The cascade: a bigger same-rank group implies the smaller ones. Two Pair is deliberately
    // left out of it, which is why five of a kind contains a Pair but not a Two Pair.
    if out.contains(HandType::FiveOfAKind) { out.insert(HandType::FourOfAKind); }
    if out.contains(HandType::FourOfAKind) { out.insert(HandType::ThreeOfAKind); }
    if out.contains(HandType::ThreeOfAKind) { out.insert(HandType::Pair); }

    out
}

/// A decided hand: its name and the cards that score for it.
///
/// `contained` is left empty on purpose. Working out every hand the cards *also* contain is a
/// separate question with different rules, and [`evaluate_hand`] fills it in afterwards.
fn hand(hand_type: HandType, scoring_indices: Vec<usize>) -> HandEvalResult {
    HandEvalResult {
        hand_type,
        scoring_indices,
        contained: ContainedHands::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardInstance;
    use crate::types::{Rank, Suit};

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
