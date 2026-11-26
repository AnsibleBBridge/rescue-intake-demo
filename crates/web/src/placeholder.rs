//! Photo-less placeholder art, one per species `Family`: an outline for the fact sheet and
//! share image (100x100 box, shared as path data) and an emoji for the roster and profile.
use rescue_core::model::Family;

/// Silhouette outline (100x100 viewBox) as SVG path data; filled, so no strokes are needed.
pub fn sil_path(family: Family) -> &'static str {
    match family {
        // Horse head, facing right.
        Family::Equine => "M30 92 L33 60 Q28 40 40 26 L40 8 L50 20 Q66 18 78 32 Q90 46 92 62 Q94 74 84 76 Q74 76 70 66 Q64 70 62 92 Z",
        // Goat face: swept-back horns, side ears, beard.
        Family::SmallRuminant => "M34 36 Q18 30 10 14 Q22 16 34 26 Q42 22 50 22 Q58 22 66 26 Q78 16 90 14 Q82 30 66 36 Q86 38 94 48 Q80 52 68 46 Q70 62 62 76 Q60 86 54 96 L50 90 L46 96 Q40 86 38 76 Q30 62 32 46 Q20 52 6 48 Q14 38 34 36 Z",
        // Chicken, side view: comb, beak, tail, legs.
        Family::Poultry => "M30 44 Q28 34 36 30 Q36 20 42 22 Q44 14 48 20 Q52 14 54 22 Q58 22 58 30 L68 34 L58 38 Q60 44 66 44 Q78 40 84 22 Q94 44 82 64 Q72 80 56 82 L56 94 L62 96 L48 96 L50 82 Q34 78 28 62 Q24 52 30 44 Z",
        // Rabbit sitting up: tall ears, round body.
        Family::SmallMammal => "M36 8 Q46 4 48 24 Q48 32 50 36 Q52 32 52 24 Q54 4 64 8 Q70 14 62 38 Q70 44 72 54 Q84 62 82 80 Q80 94 62 94 L38 94 Q20 94 18 80 Q16 62 28 54 Q30 44 38 38 Q30 14 36 8 Z",
        Family::Dog => "M26 24 Q38 22 42 34 Q50 32 58 34 Q62 22 74 24 Q88 30 82 56 Q76 52 74 46 Q76 70 62 82 Q50 92 38 82 Q24 70 26 46 Q24 52 18 56 Q12 30 26 24 Z",
        Family::Cat => "M22 14 L40 32 Q50 29 60 32 L78 14 L80 48 Q86 60 78 74 Q66 90 50 90 Q34 90 22 74 Q14 60 20 48 Z",
        // Paw print: pad plus four toes.
        Family::Other => "M50 52 Q72 52 76 72 Q78 90 62 90 Q54 88 50 88 Q46 88 38 90 Q22 90 24 72 Q28 52 50 52 Z M12 46 a9 11 0 1 0 18 0 a9 11 0 1 0 -18 0 Z M30 26 a10 12 0 1 0 20 0 a10 12 0 1 0 -20 0 Z M50 26 a10 12 0 1 0 20 0 a10 12 0 1 0 -20 0 Z M70 46 a9 11 0 1 0 18 0 a9 11 0 1 0 -18 0 Z",
    }
}

/// Emoji stand-in on cards where a full silhouette would be too heavy.
pub fn emoji(family: Family) -> &'static str {
    match family {
        Family::Equine => "\u{1F434}",
        Family::SmallRuminant => "\u{1F410}",
        Family::Poultry => "\u{1F414}",
        Family::SmallMammal => "\u{1F430}",
        Family::Dog => "\u{1F436}",
        Family::Cat => "\u{1F431}",
        Family::Other => "\u{1F43E}",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_has_art() {
        for f in [Family::Equine, Family::SmallRuminant, Family::Poultry, Family::SmallMammal, Family::Dog, Family::Cat, Family::Other] {
            assert!(sil_path(f).starts_with('M') && !emoji(f).is_empty());
        }
    }
}
