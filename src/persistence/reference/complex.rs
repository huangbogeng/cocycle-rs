//! Simplices required by the initial H0/H1 computation.

/// Vertices in an edge or triangle must be strictly increasing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Simplex {
    Vertex(usize),
    Edge([usize; 2]),
    Triangle([usize; 3]),
}

impl Simplex {
    pub(crate) fn dimension(self) -> usize {
        match self {
            Self::Vertex(_) => 0,
            Self::Edge(_) => 1,
            Self::Triangle(_) => 2,
        }
    }

    /// Over F2 the boundary is the set of codimension-one faces, without signs.
    pub(crate) fn faces(self) -> [Option<Self>; 3] {
        match self {
            Self::Vertex(_) => [None, None, None],
            Self::Edge([a, b]) => [Some(Self::Vertex(a)), Some(Self::Vertex(b)), None],
            Self::Triangle([a, b, c]) => [
                Some(Self::Edge([a, b])),
                Some(Self::Edge([a, c])),
                Some(Self::Edge([b, c])),
            ],
        }
    }

    pub(crate) fn is_valid(self) -> bool {
        match self {
            Self::Vertex(_) => true,
            Self::Edge([a, b]) => a < b,
            Self::Triangle([a, b, c]) => a < b && b < c,
        }
    }
}
