import sys
from dataclasses import dataclass
from pathlib import Path
from time import time as cur_time
from typing import List, Set

import click
from specmonkey_native import extract_links

# Define the supported file extensions
SUPPORTED_EXTENSIONS = {".cpp", ".h", ".rs", ".js", ".html"}


@dataclass
class Link:
    url: str
    line_number: int
    file_name: str


def gather_code_files(
    directory: Path, extensions: Set[str], verbose: bool = False
) -> List[Path]:
    """
    Recursively gather all code files with supported extensions from the given directory.

    Args:
        directory (Path): The root directory to scan.
        extensions (Set[str]): Set of file extensions to include.
        verbose (bool): If True, print each file as it's found.

    Returns:
        List[Path]: List of full file paths.
    """
    code_files = []
    for file_path in directory.rglob("*"):
        if file_path.is_file() and file_path.suffix.lower() in extensions:
            code_files.append(file_path)
            if verbose:
                click.echo(f"Found file: {file_path}")
    return code_files


def write_links_to_file(
    links: List[Link], output_file: Path, format: str = "txt", verbose: bool = False
):
    """
    Write the extracted links to the output file.

    Args:
        links (List[Link]): List of Link objects containing url, line_number, and file_name.
        output_file (Path): Path to the output text file.
        format (str): Output file format ('txt', 'csv', 'json').
        verbose (bool): If True, print each link as it's written.
    """
    if format == "txt":
        with output_file.open("w", encoding="utf-8") as f:
            for link in links:
                line = f"{link.file_name}:{link.line_number}:{link.url}\n"
                f.write(line)
                if verbose:
                    click.echo(f"Wrote link: {line.strip()}")
    elif format == "csv":
        import csv

        with output_file.open("w", encoding="utf-8", newline="") as f:
            writer = csv.writer(f)
            writer.writerow(["filename", "line_number", "url"])
            for link in links:
                writer.writerow([link.file_name, link.line_number, link.url])
                if verbose:
                    click.echo(
                        f"Wrote link: {link.file_name},{link.line_number},{link.url}"
                    )
    elif format == "json":
        import json

        json_data = [
            {
                "filename": link.file_name,
                "line_number": link.line_number,
                "url": link.url,
            }
            for link in links
        ]
        with output_file.open("w", encoding="utf-8") as f:
            json.dump(json_data, f, indent=4)
            if verbose:
                click.echo(f"Wrote {len(json_data)} links to {output_file}")
    else:
        click.echo(f"Unsupported format: {format}")
        sys.exit(1)


@click.group()
def cli():
    """
    SpecMonkey CLI - A tool to extract URLs from code comments.
    """
    pass


@cli.command(name="simple-list")
@click.argument(
    "directory", type=click.Path(exists=True, file_okay=False, readable=True)
)
@click.argument("output_file", type=click.Path(), required=False, default="output.txt")
@click.option(
    "--format",
    "-f",
    type=click.Choice(["txt", "csv", "json"], case_sensitive=False),
    default="txt",
    help="Output file format. Defaults to 'txt'.",
)
@click.option("--verbose", is_flag=True, help="Enables verbose mode.")
@click.option(
    "--extensions",
    "-e",
    multiple=True,
    default=[".cpp", ".h", ".rs", ".js", ".html"],
    help="File extensions to include. Defaults to .cpp, .h, .rs, .js, .html",
)
def simple_list(
    directory: str, output_file: str, format: str, verbose: bool, extensions: tuple
):
    """
    Scan DIRECTORY for code files, extract URLs from comments, and write to OUTPUT_FILE.

    DIRECTORY: The root directory to scan for code files.
    OUTPUT_FILE: (Optional) The file to write the extracted links. Defaults to 'output.txt'.
    """
    directory_path = Path(directory)
    output_path = Path(output_file)
    start_time = cur_time()
    click.echo(f"Scanning directory: {directory_path}")
    click.echo(f"Output will be saved to: {output_path}")
    if verbose:
        click.echo("Verbose mode enabled.")

    time = cur_time()
    # Gather all relevant code files
    code_files = gather_code_files(directory_path, set(extensions), verbose=verbose)
    if not code_files:
        click.echo("No code files found with supported extensions.")
        sys.exit(0)

    click.echo(
        f"Found {len(code_files)} code file(s) to process "
        f"(elapsed time: {cur_time() - time} s)."
    )
    time = cur_time()
    # Extract links using the Rust-native function
    links = extract_links([str(file) for file in code_files])
    if not links:
        click.echo("No links found in the scanned files.")
    else:
        click.echo(
            f"Extracted {len(links)} link(s) (elapsed time: {cur_time() - time} s)."
        )
        # Convert extracted links to the Link dataclass
        link_objects = [
            Link(link.url, link.line_number, link.file_name) for link in links
        ]
        # Write the links to the output file
        time = cur_time()
        write_links_to_file(link_objects, output_path, format=format, verbose=verbose)
        click.echo(
            f"Links have been written to {output_path} (elapsed time: {cur_time() - time} s)."
        )
        click.echo(f"Total elapsed time: {cur_time() - start_time} s.")


if __name__ == "__main__":
    cli()
