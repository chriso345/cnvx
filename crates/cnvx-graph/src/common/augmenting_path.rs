/// Runs BFS from every node in `sources` simultaneously over the graph
/// implied by `neighbors`, returning each node's layer (`0` for a source,
/// `-1` if unreached).
pub(crate) fn bfs_layers(
    n: usize,
    sources: &[usize],
    mut neighbors: impl FnMut(usize, &mut Vec<usize>),
) -> Vec<i32> {
    let mut layer = vec![-1i32; n];
    let mut queue = std::collections::VecDeque::new();
    for &s in sources {
        if layer[s] == -1 {
            layer[s] = 0;
            queue.push_back(s);
        }
    }
    let mut scratch = Vec::new();
    while let Some(u) = queue.pop_front() {
        scratch.clear();
        neighbors(u, &mut scratch);
        for &v in &scratch {
            if layer[v] == -1 {
                layer[v] = layer[u] + 1;
                queue.push_back(v);
            }
        }
    }
    layer
}
