# fuzzy-ls
![crates.io](https://img.shields.io/crates/v/fuzzy-ls.svg) ![Build Passing](https://github.com/Ashwin-1709/fuzzy-ls/actions/workflows/rust.yml/badge.svg)

`fuzzy-ls` is a cross-platform command line utility that extends the functionality of the popular `ls` command by enabling fuzzy searching, regex pattern matching, exact matches, and more. It allows you to focus your search on specific file extensions or exclude certain extensions from the search space.

## Features

- **Fuzzy Searching**: Perform fuzzy searches to find files that approximately match your query.
- **Regex Pattern Matching**: Use regular expressions to search for files.
- **Exact Matches**: Search for files that exactly match your query.
- **Focus on Extensions**: Limit your search to specific file extensions.
- **Exclude Extensions**: Exclude files with certain extensions from your search.

## Fuzzy Searching Algorithm

Currently, the tool uses the [Damerau-Levenshtein](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance) algorithm for fuzzy searching. The Damerau-Levenshtein algorithm calculates the minimum number of operations (insertions, deletions, substitutions, and transpositions) required to transform one string into another. There are plans to add more scorers later to enhance the search capabilities.

## Usage

The help menu contains the necessary documentation on different options supported.

```
Fuzzy file search command line tool.

Usage: fuzzy-ls.exe [OPTIONS] <QUERY>


Options:
  -r, --regex                Query is a regex pattern and the search is performed using the regex.
  -p, --exact                Exact pattern matching is done for the query.
  -e, --exclude [<.ext>...]  Exclude files of specific extensions.
  -f, --focus [<.ext>...]    Focus search on specific set of extensions. In case both exclude and focus are provided, focus takes precedence.
  -h, --help                 Print help
  -V, --version              Print versionfuzzy-ls.exe [OPTIONS] <QUERY>
```
## Examples
Here are some examples demonstrating how to use `fuzzy-ls`. Exact matches are highlighted by green on terminal and fuzzy matches with blue like this:

![fuzzy_search](static/fuzzy_search.png)

### Fuzzy search
```shell
fuzzy-ls search
search - .\src\search.rs
searchfn - .\test_data\searchfn.java
```

### Regex search
```shell
fuzzy-ls fuzzy.* -r
fuzzy_ls - .\target\debug\deps\fuzzy_ls.d
fuzzy_ls - .\target\debug\deps\fuzzy_ls.exe
fuzzy_ls - .\target\debug\deps\fuzzy_ls.pdb
fuzzy-ls - .\target\debug\fuzzy-ls.d
fuzzy-ls - .\target\debug\fuzzy-ls.exe
fuzzy_ls - .\target\debug\fuzzy_ls.pdb
fuzzy_search - .\test_data\fuzzy_search.png
```

You can mix and match focused flags with any of the search type

#### Regex with focused extensions
```shell
fuzzy-ls fuzzy.* -r -f exe
fuzzy_ls-2f8732b3ab03d5a6 - .\target\debug\deps\fuzzy_ls-2f8732b3ab03d5a6.exe
fuzzy_ls-c33a3a1ef73b944b - .\target\debug\deps\fuzzy_ls-c33a3a1ef73b944b.exe
fuzzy_ls - .\target\debug\deps\fuzzy_ls.exe
fuzzy-ls - .\target\debug\fuzzy-ls.exe
```

#### Regex with exclude extensions
```shell
fuzzy-ls fuzzy.* -r -e exe
fuzzy_ls - .\target\debug\deps\fuzzy_ls.d
fuzzy_ls - .\target\debug\deps\fuzzy_ls.pdb
fuzzy-ls - .\target\debug\fuzzy-ls.d
fuzzy_ls - .\target\debug\fuzzy_ls.pdb
fuzzy_search - .\test_data\fuzzy_search.png
```

Note: In case both focused and exclude extensions are provided: focus extensions take a precedence.

### Exact string matching
```shell
fuzzy-ls utils -p
utils - .\test_data\utils.cpp
utils - .\test_data\utils.h
```

## Experimental Feature: Open files in editor
The `open_in_editor` feature allows you to open files directly in your preferred editor from the command line. By default, this feature uses `nvim` (Neovim) and opens the file in a new terminal window. You can override the default editor using the `-d` flag.

### Installation and Usage

To install and run the feature locally, use the following command:
```shell
cargo install fuzzy-ls --features "open_in_editor"
```

### Here is the feature in action in windows

![nvim](static/code_editor_nvim.png)


To override the default editor, use the `-d` or `--default_editor_command` flag followed by the editor of your choice. For example, to use VS Code:
```shell
fuzzy-ls search -d code 
```

![vscode](static/code_editor_vscode.png)

### Platform Support

The `open_in_editor` feature has been tested on Windows. Testing on other operating systems is planned before merging with the default behavior. Users are encouraged to try it out on various OSes and report any issues.


## Community Contribution
Feel free to contribute to the project whether its reporting issues/suggestions or feature requests or new feature PRs!

## License
This project is licensed under the MIT License.
