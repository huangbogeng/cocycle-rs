//! Public contracts for graph.

use cocycle::complex::{WeightedEdge as Edge, WeightedGraph as Graph};

#[test]
fn adjacency_preserves_zero_edges_and_isolated_vertices() {
    let graph = Graph::new(
        5,
        vec![
            Edge {
                vertices: [3, 1],
                value: 2.,
            },
            Edge {
                vertices: [1, 0],
                value: -0.,
            },
            Edge {
                vertices: [3, 0],
                value: 4.,
            },
        ],
    )
    .unwrap();
    assert_eq!(graph.vertex_count(), 5);
    assert_eq!(graph.edge_count(), 3);
    assert_eq!(
        graph.edges().iter().map(|e| e.vertices).collect::<Vec<_>>(),
        vec![[0, 1], [0, 3], [1, 3]]
    );
    assert_eq!(
        graph
            .neighbors(0)
            .unwrap()
            .iter()
            .map(|n| n.vertex)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert!(graph.neighbors(4).unwrap().is_empty());
    assert!(graph.neighbors(5).is_none());
    assert_eq!(graph.edge_value(0, 1).unwrap().to_bits(), 0);
    assert_eq!(graph.edge_value(3, 1), Some(2.));
    for pair in [(0, 0), (0, 2), (0, 5), (5, 0)] {
        assert_eq!(graph.edge_value(pair.0, pair.1), None);
    }
}

#[test]
fn invalid_graphs_are_never_repaired_silently() {
    for edge in [
        Edge {
            vertices: [0, 0],
            value: 0.,
        },
        Edge {
            vertices: [0, 2],
            value: 0.,
        },
        Edge {
            vertices: [0, 1],
            value: -1.,
        },
        Edge {
            vertices: [0, 1],
            value: f64::INFINITY,
        },
    ] {
        assert!(Graph::new(2, vec![edge]).is_err());
    }
    assert!(
        Graph::new(
            2,
            vec![
                Edge {
                    vertices: [0, 1],
                    value: 1.
                },
                Edge {
                    vertices: [1, 0],
                    value: 2.
                }
            ]
        )
        .is_err()
    );
    assert!(Graph::new(usize::MAX, vec![]).is_err());
    assert_eq!(Graph::new(0, vec![]).unwrap().vertex_count(), 0);
}
