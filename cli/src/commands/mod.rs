use clap::Subcommand;

pub mod add;
pub mod cat;
pub mod checkout;
pub mod common;
pub mod diff;
pub mod edit;
pub mod export;
pub mod get;
pub mod import;
pub mod ipc;
pub mod link;
pub mod meta;
pub mod mount;
pub mod policy;
pub mod rename;
pub mod revise;
pub mod revoke;
pub mod share;
pub mod shares;
pub mod state;
pub mod stats;
pub mod system;
pub mod tag;
pub mod tags;
pub mod trust;
pub mod untag;
pub mod versions;
pub mod view;
pub mod watchd;

#[derive(Subcommand, Debug)]
pub enum Command {
    Add(add::AddArgs),
    Tag(tag::TagArgs),
    Tags(tags::TagsArgs),
    Untag(untag::UntagArgs),
    Rename(rename::RenameArgs),
    Link(link::LinkArgs),
    Meta(meta::MetaArgs),
    Get(get::GetArgs),
    Cat(cat::CatArgs),
    Versions(versions::VersionsArgs),
    Diff(diff::DiffArgs),
    Restore(versions::RestoreArgs),
    Checkout(checkout::CheckoutArgs),
    Revise(revise::ReviseArgs),
    State(state::StateArgs),
    View(view::ViewArgs),
    Share(share::ShareCommand),
    Revoke(revoke::RevokeArgs),
    Shares(shares::SharesArgs),
    Policy(policy::PolicyArgs),
    Trust(trust::TrustArgs),
    Quarantine(trust::QuarantineArgs),
    Stats(stats::StatsArgs),
    Import(import::ImportArgs),
    Export(export::ExportArgs),
    Edit(edit::EditArgs),
    Watchd(watchd::WatchdArgs),
    Mount(mount::MountArgs),
    Unmount(mount::UnmountArgs),
    Init(system::InitArgs),
    Status(system::StatusArgs),
    Gc(system::GcArgs),
    Verify(system::VerifyArgs),
    Ipc(ipc::IpcArgs),
}
