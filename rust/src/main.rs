use anyhow::Result;

fn main() -> Result<()> {
    //in rust, a crate in kebab-case is imported in snake_case
    //rust can use crate name as implicit root module, so we can directly call the function without importing it
    //root module can chain to submodules, so we can directly call the function without importing it
    dev_purge::core::run()
}
