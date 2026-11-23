/// Wires `$solver` up to `cnvx_core::Solve` for both `&Graph<N, E>` and
/// `&GraphView<'_, N, E>`, delegating to `$func`, a free function generic
/// over `G: GraphRef<N, E> + Copy`.
///
/// ```ignore
/// impl_solve!(Bfs => BfsSolution, bfs_impl);
/// ```
macro_rules! impl_solve {
    ($solver:ty => $solution:ty, $func:path) => {
        impl<N, E> cnvx_core::Solve<$solver> for &crate::Graph<N, E> {
            type Solution = $solution;

            fn solve(&self, solver: &$solver) -> Result<$solution, cnvx_core::CnvxError> {
                $func(*self, solver)
            }
        }
        impl<N, E> cnvx_core::Solve<$solver> for &crate::GraphView<'_, N, E> {
            type Solution = $solution;

            fn solve(&self, solver: &$solver) -> Result<$solution, cnvx_core::CnvxError> {
                $func(*self, solver)
            }
        }
    };
}

pub(crate) use impl_solve;
