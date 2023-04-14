use derive_more::{Display, From, FromStr, Into};
use diagnostics::errors::syntax::SyntaxError;
use getset::{Getters, MutGetters, Setters};
use lazy_static::lazy_static;
use miette::{IntoDiagnostic, Result, SourceSpan};
use regex::Regex;
use shrinkwraprs::Shrinkwrap;
use smartstring::alias::String;
use typed_builder::TypedBuilder;

lazy_static! {
    /// A regular expression used to validate a label's repository name.
    static ref LABEL_REPO_RE: Regex = Regex::new(r"^@[\w\-.][\w\-.~]*$").unwrap();
}

/// A Repo represents a **repository** (e.g. `@foo`, `@fbcode`, `@com_github_foo_bar`).
/// Repositories are used to group related packages together. A repository is identified
/// by a name, which must be a valid label repository name (i.e. it must start with an `@`
/// and contain only alphanumeric characters and underscores).
#[derive(
    Debug,
    Default,
    Display,
    Clone,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    From,
    Into,
    FromStr,
    Shrinkwrap,
)]
pub struct Repo(String);

impl Repo {
    pub fn new(label: &str, repo: &str) -> Result<Repo> {
        Self::parse(label, repo)
    }

    pub fn parse(label: &str, repo: &str) -> Result<Repo> {
        if !LABEL_REPO_RE.is_match(repo) || !label.contains(repo) {
            return Err(SyntaxError::InvalidRepoName {
                label: label.into(),
                repo: repo.into(),
                span: SourceSpan::new(0.into(), repo.len().into()),
            })
            .into_diagnostic();
        }

        Ok(Repo(repo.into()))
    }
}

/// A Label represents a **label** of a **build target**. Labels have three
/// parts: a _repository name_, a _package name_, and a _target name_, formatted
/// as **@repo//pkg:target**.
#[derive(
    Debug,
    Default,
    Display,
    Clone,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Getters,
    MutGetters,
    Setters,
    TypedBuilder,
)]
#[getset(get = "pub", get_mut = "pub", set = "pub")]
#[display(fmt = "@{repo}//{pkg}:{name}")]
pub struct Label {
    /// Repo is the repository name. If omitted, the label refers to a target
    /// in the current repository.
    #[builder(setter(into))]
    pub repo: Repo,

    /// Pkg is the package name, which is usually the directory that contains
    /// the target. If both Repo and Pkg are omitted, the label is relative.
    #[builder(setter(into))]
    pub pkg: String,

    /// Name is the name of the target the label refers to. Name must not be empty.
    /// Note that the name may be omitted from a label string if it is equal to
    /// the last component of the package name ("//x" is equivalent to "//x:x"),
    /// but in either case, Name should be set here.
    #[builder(setter(into))]
    pub name: String,

    /// Relative indicates whether the label refers to a target in the current
    /// package. Relative is true iff repo and pkg are both omitted
    /// (i.e. `:name` instead of `//pkg:name`)
    #[builder(default, setter(into))]
    pub relative: bool,
}

impl Label {
    /// Creates a new label with the given `repo`, `package`, and `name`.
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository name. If omitted, the label refers to a target
    /// in the current repository.
    ///
    /// * `pkg` - The package name, which is usually the directory that contains
    /// the target. If both Repo and Pkg are omitted, the label is relative.
    ///
    /// * `name` - The name of the target the label refers to. Name must not be empty.
    /// Note that the name may be omitted from a label string if it is equal to
    /// the last component of the package name ("//x" is equivalent to "//x:x"),
    /// but in either case, Name should be set here.
    ///
    /// # Examples
    ///
    /// ```
    /// let label = Label::new("@foo//bar:baz").expect("failed to create label");
    ///
    /// assert_eq!(label.repo(), "@foo");
    /// assert_eq!(label.pkg(), "bar");
    /// assert_eq!(label.name(), "baz");
    /// ```
    pub fn new(label: &str) -> Result<Label> {
        // split the string on the // delimiter to get the unchecked repo
        let unchecked_repo: &str = label.split("//").next().ok_or(SyntaxError::InvalidLabel {
            label: label.into(),
            span: SourceSpan::new(0.into(), label.len().into()),
        })?;

        let repo = Repo::new(label, unchecked_repo)?;

        todo!("Label::new")

        // Label {
        //     repo: Repo::new(repo).into_diagnostic().map_err(|e| e.exit()),
        //     pkg: pkg.into(),
        //     name: name.into(),
        //     relative: false,
        // }
    }

    /// Parses a label from a string. The string must be in the format
    /// **@repo//pkg:target**.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// let label = Label::parse("@foo//bar:baz").expect("failed to parse label");
    ///
    /// assert_eq!(label.repo(), "@foo");
    /// assert_eq!(label.pkg(), "bar");
    /// assert_eq!(label.name(), "baz");
    /// ```
    fn parse(s: &str) -> Result<Label> {
        todo!()
        // let mut repo = String::new();
        // let mut pkg = String::new();
        // let mut name = String::new();

        // let mut parts = s.split("//");
        // let repo_part = parts.next().unwrap();
        // let pkg_part = parts.next().unwrap_or_default();

        // if !repo_part.is_empty() {
        //     if !LABEL_REPO_RE.is_match(repo_part) {
        //         return Err(format!("Invalid repo name: {}", repo_part));
        //     }

        //     repo = repo_part.to_string();
        // }

        // let mut parts = pkg_part.split(":");
        // let mut pkg_part = parts.next().unwrap();
        // let mut name_part = parts.next().unwrap_or_default();

        // if !pkg_part.is_empty() {
        //     if !LABEL_PKG_RE.is_match(pkg_part) {
        //         return Err(format!("Invalid package name: {}", pkg_part));
        //     }

        //     pkg = pkg_part.to_string();
        // }

        // if !name_part.is_empty() {
        //     if !LABEL_NAME_RE.is_match(name_part) {
        //         return Err(format!("Invalid target name: {}", name_part));
        //     }

        //     name = name_part.to_string();
        // }

        // Ok(Label {
        //     repo,
        //     pkg,
        //     name,
        //     relative: repo.is_empty() && pkg.is_empty(),
        // })
    }
}
