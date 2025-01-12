#!/usr/bin/env python3

import json
import os
import shutil
import subprocess
import tempfile
import zipfile
from pathlib import Path


def get_project_root():
    """
    Returns the root directory of the project, assuming this script is in /scripts/.
    """
    return Path(__file__).resolve().parent.parent


def extract_domains(config_path):
    """
    Extracts the list of domains from config.json.

    Args:
        config_path (Path): Path to config.json.

    Returns:
        list: List of domains.
    """
    try:
        with config_path.open("r", encoding="utf-8") as f:
            config = json.load(f)
        domains = config.get("domains")
        if not isinstance(domains, list):
            raise ValueError("'domains' should be a list in config.json.")
        return domains
    except Exception as e:
        raise Exception(f"Error reading {config_path}: {e}") from e


def update_manifest(
    manifest_in_path: Path,
    domains: list[str],
    version: str,
    temp_manifest_path: Path,
):
    """
    Updates the manifest.in.json with new domains and version, and saves as manifest.json.

    Args:
        manifest_in_path (Path): Path to manifest.in.json.
        domains (list): List of domains to set in content_scripts.matches.
        version (str): Version string to set in manifest.
        temp_manifest_path (Path): Path to save the updated manifest.json.
    """
    try:
        with manifest_in_path.open("r", encoding="utf-8") as f:
            manifest = json.load(f)

        # Update content_scripts.matches
        content_scripts = manifest.get("content_scripts", [])
        if not content_scripts:
            raise ValueError(
                "'content_scripts' not found or empty in manifest.in.json."
            )

        content_scripts[0]["matches"] = [f"*://*.{domain}/*" for domain in domains]

        # Update version
        manifest["version"] = version

        # Save to temp_manifest_path
        with temp_manifest_path.open("w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)

        print(f"Updated manifest.json saved to {temp_manifest_path}")
    except Exception as e:
        raise Exception(f"Error updating manifest: {e}") from e


def get_latest_git_tag() -> str:
    """
    Retrieves the latest Git tag.

    Returns:
        str: Latest Git tag.
    """
    try:
        tag = subprocess.check_output(
            ["git", "describe", "--tags", "--abbrev=0"], stderr=subprocess.STDOUT
        )
        return tag.decode("utf-8").strip()
    except subprocess.CalledProcessError as e:
        msg = (
            "Error retrieving the latest Git tag. "
            "Ensure that the repository has at least one tag.\n"
        )
        msg += e.output.decode()
        raise Exception(msg) from e


def create_zipfile(
    source_dir: Path,
    manifest_path: Path,
    output_zip_path: Path,
) -> None:
    """
    Creates a ZIP archive of the extension.

    Args:
        source_dir (Path): Path to the /extension directory.
        manifest_path (Path): Path to the updated manifest.json.
        output_zip_path (Path): Path to save the ZIP file.
    """
    try:
        with zipfile.ZipFile(output_zip_path, "w", zipfile.ZIP_DEFLATED) as zipf:
            for root, _, files in os.walk(source_dir):
                for file in files:
                    file_path = Path(root) / file
                    relative_path = file_path.relative_to(source_dir)
                    if relative_path.name == "manifest.in.json":
                        continue  # Exclude manifest.in.json
                    zipf.write(file_path, arcname=relative_path)
            # Add the updated manifest.json
            zipf.write(manifest_path, arcname="manifest.json")
        print(f"Extension packaged successfully at {output_zip_path}")
    except Exception as e:
        raise Exception(f"Error creating ZIP file: {e}") from e


def main():
    # Define paths
    project_root = get_project_root()
    extension_dir = project_root / "extension"
    config_path = extension_dir / "config.json"
    manifest_in_path = extension_dir / "manifest.in.json"
    temp_dir = Path(tempfile.mkdtemp())
    temp_manifest_path = temp_dir / "manifest.json"
    output_dir = project_root / "dist"
    output_dir.mkdir(exist_ok=True)

    # Step 1: Extract domains from config.json
    domains = extract_domains(config_path)
    print(f"Extracted domains: {domains}")

    # Step 2: Get latest Git tag
    latest_tag = get_latest_git_tag()
    print(f"Latest Git tag: {latest_tag}")

    # Step 3: Update manifest and save as manifest.json in temp_dir
    update_manifest(manifest_in_path, domains, latest_tag, temp_manifest_path)

    # Step 4: Create ZIP archive
    # Define the output ZIP file name, e.g., specmonkey_v1.0.0.zip
    zip_filename = f"specmonkey_{latest_tag}.zip"
    output_zip_path = output_dir / zip_filename

    create_zipfile(extension_dir, temp_manifest_path, output_zip_path)

    # Clean up temp_dir
    shutil.rmtree(temp_dir)
    print(f"Temporary files cleaned up from {temp_dir}")


if __name__ == "__main__":
    main()
