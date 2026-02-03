use clap::Subcommand;

pub mod add;
pub mod common;
pub mod cat;
pub mod checkout;
pub mod diff;
pub mod export;
pub mod get;
pub mod import;
pub mod link;
pub mod mount;
pub mod policy;
pub mod revoke;
pub mod revise;
pub mod share;
pub mod shares;
pub mod system;
pub mod tag;
pub mod tags;
pub mod trust;
pub mod untag;
pub mod versions;
pub mod view;

#[derive(Subcommand, Debug)]
pub enum Command {
    Add(add::AddArgs),
    Tag(tag::TagArgs),
    Tags(tags::TagsArgs),
    Untag(untag::UntagArgs),
    Link(link::LinkArgs),
    Get(get::GetArgs),
    Cat(cat::CatArgs),
    Versions(versions::VersionsArgs),
    Diff(diff::DiffArgs),
    Restore(versions::RestoreArgs),
    Checkout(checkout::CheckoutArgs),
    Revise(revise::ReviseArgs),
    View(view::ViewArgs),
    Share(share::ShareCommand),
    Revoke(revoke::RevokeArgs),
    Shares(shares::SharesArgs),
    Policy(policy::PolicyArgs),
    Trust(trust::TrustArgs),
    Quarantine(trust::QuarantineArgs),
    Import(import::ImportArgs),
    Export(export::ExportArgs),
    Mount(mount::MountArgs),
    Unmount(mount::UnmountArgs),
    Init(system::InitArgs),
    Status(system::StatusArgs),
    Gc(system::GcArgs),
    Verify(system::VerifyArgs),
}
