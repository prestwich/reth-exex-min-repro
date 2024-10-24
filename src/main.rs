use futures_util::StreamExt;
use reth_exex::ExExContext;
use reth_node_api::FullNodeComponents;
use std::fmt::Debug;

struct ExEx<Host>
where
    Host: FullNodeComponents,
{
    ctx: ExExContext<Host>,
}

impl<Host> ExEx<Host>
where
    Host: FullNodeComponents,
{
    async fn start(self) -> eyre::Result<()> {
        // `.provider()` requires a debug bound on `Host::Provider`
        // so is inacessible without that bound
        let _provider = self.ctx.provider();
        while let Some(_notification) = self.ctx.notifications.next().await {}
        Ok(())
    }

    async fn start_with_bounds(mut self) -> eyre::Result<()>
    where
        Host: FullNodeComponents<Provider: Debug, Executor: Debug>,
    {
        // Can call the `.provider()` method as the `Host` type has the
        // required bounds.
        // However, we can't call `.start()` from within `install_exex`
        let _provider = self.ctx.provider();
        while let Some(_notification) = self.ctx.notifications.next().await {}
        Ok(())
    }
}

fn main() {
    reth::cli::Cli::parse_args()
        .run(|builder, _| async move {
            let db_args = reth_db::mdbx::DatabaseArguments::default();

            let handle = builder
                .node(reth_node_ethereum::EthereumNode::default())
                // this works, but disables access to key `ctx` methods
                .install_exex("without bound", move |ctx| async {
                    Ok(ExEx { ctx }.start())
                })
                // this doesn't work as the `install_exex` method doesn't place
                // the required bound
                .install_exex("with bound", move |ctx| async {
                    Ok(ExEx { ctx }.start_with_bounds())
                })
                .launch()
                .await?;

            handle.wait_for_node_exit().await
        })
        .unwrap()
}
