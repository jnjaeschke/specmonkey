from pathlib import Path

import pytest
from specmonkey_native import extract_links


@pytest.fixture(scope="module")
def test_files_dir():
    """
    Fixture to provide the path to the test_files directory.
    """
    return Path(__file__).parent / "test_files"


@pytest.fixture(scope="module")
def file_paths(test_files_dir):
    """
    Fixture to provide a list of full paths to the test files.
    """
    return [
        str(test_files_dir / "file1.cpp"),
        str(test_files_dir / "file2.h"),
        str(test_files_dir / "file3.rs"),
        str(test_files_dir / "file4.js"),
        str(test_files_dir / "file5.html"),
    ]


def test_extract_links(file_paths, test_files_dir):
    """
    Test the extract_links function with actual files.

    Args:
        file_paths (list of str): List of file paths to process.

    Verifies that the returned Link objects contain the correct URLs, line numbers, and file names.
    """
    # Call the Rust extension function to extract links
    links = extract_links(file_paths)

    # Define expected links as a set of tuples (url, line_number, file_name)
    expected_links = {
        ("https://example.com/cpp", 2, str(test_files_dir / "file1.cpp")),
        (
            "http://foo.specs.open.org/folder/#section-2",
            3,
            str(test_files_dir / "file1.cpp"),
        ),
        ("https://api.specmonkey.com/v1", 5, str(test_files_dir / "file4.js")),
        (
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo",
            2,
            str(test_files_dir / "file2.h"),
        ),
        (
            "https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah",
            3,
            str(test_files_dir / "file2.h"),
        ),
        ("https://rust-lang.org", 2, str(test_files_dir / "file3.rs")),
        (
            "https://developer.mozilla.org/en-US/docs/Web/JavaScript",
            2,
            str(test_files_dir / "file4.js"),
        ),
        (
            "http://foo.specs.open.org/folder/#section-2",
            3,
            str(test_files_dir / "file4.js"),
        ),
        (
            "https://foo.specs.open.org/folder/#section-2",
            10,
            str(test_files_dir / "file5.html"),
        ),
        (
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo",
            3,
            str(test_files_dir / "file5.html"),
        ),
        (
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo",
            14,
            str(test_files_dir / "file5.html"),
        ),
        ("https://example.com/html", 2, str(test_files_dir / "file5.html")),
    }

    # Extract actual links as a set of tuples
    actual_links = set((link.url, link.line_number, link.file_name) for link in links)

    # Compare the expected and actual links
    assert (
        actual_links == expected_links
    ), "Extracted links do not match expected links."


def test_extract_links_no_urls():
    """
    Test that passing an empty list of file paths returns an empty list of links.

    Verifies that no links are extracted when no files are provided.
    """
    links = extract_links([])
    assert (
        links == []
    ), "Extracted links should be empty when no file paths are provided."


def test_extract_links_nonexistent_file(test_files_dir):
    """
    Test that nonexistent files are handled gracefully.

    Args:
        test_files_dir (str): Path to the test_files directory.

    Verifies that attempting to extract links from a nonexistent file
    does not raise an error and is skipped.
    """
    nonexistent_file = str(test_files_dir / "nonexistent_file.cpp")
    links = extract_links([nonexistent_file])
    assert (
        links == []
    ), "Extracted links should be empty when processing a nonexistent file."


def test_extract_links_specific_file(test_files_dir):
    """
    Test extracting links from a single specific file.

    Args:
        test_files_dir (str): Path to the test_files directory.

    Verifies that the function correctly extracts links from one file.
    """
    file_path = str(test_files_dir / "file2.h")
    links = extract_links([file_path])

    # Define expected links for file2.h
    expected_links = {
        (
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo",
            2,
            str(test_files_dir / "file2.h"),
        ),
        (
            "https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah",
            3,
            str(test_files_dir / "file2.h"),
        ),
    }

    # Extract actual links as a set of tuples
    actual_links = set((link.url, link.line_number, link.file_name) for link in links)

    # Compare the expected and actual links
    assert (
        actual_links == expected_links
    ), "Extracted links for file2.h do not match expected links."


def test_extract_links_multiple_files(test_files_dir):
    """
    Test extracting links from multiple files simultaneously.

    Args:
        test_files_dir (str): Path to the test_files directory.

    Verifies that links from all provided files are correctly extracted.
    """
    file1 = str(test_files_dir / "file1.cpp")
    file2 = str(test_files_dir / "file2.h")
    file3 = str(test_files_dir / "file3.rs")

    links = extract_links([file1, file2, file3])

    # Define expected links
    expected_links = {
        ("https://example.com/cpp", 2, str(test_files_dir / "file1.cpp")),
        (
            "http://foo.specs.open.org/folder/#section-2",
            3,
            str(test_files_dir / "file1.cpp"),
        ),
        (
            "https://bugzilla.mozilla.org/show_bug.cgi?id=1234#foo",
            2,
            str(test_files_dir / "file2.h"),
        ),
        (
            "https://bugzil.la/5678#:~:text=foo-,bar%20baz,-blah",
            3,
            str(test_files_dir / "file2.h"),
        ),
        ("https://rust-lang.org", 2, str(test_files_dir / "file3.rs")),
    }

    # Extract actual links as a set of tuples
    actual_links = set((link.url, link.line_number, link.file_name) for link in links)

    # Compare the expected and actual links
    assert (
        actual_links == expected_links
    ), "Extracted links from multiple files do not match expected links."
