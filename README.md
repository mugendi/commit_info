This crate gathers relevant git info from any Repo. Some of the info returned includes:
- **Git status info**: Checks if a repo is dirty, has been modified and so on.
- **Commits**: Gathers and shows information for the last 10 commits

## Example

```rust

 let dir = "/path/to/repo"; //<- Point to the location of t=your repo
 let info = Info::new(&dir).status_info()?.commit_info()?;
 println("{:#?}", info);

```