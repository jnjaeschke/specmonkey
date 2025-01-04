// content.js

// Ensure the script runs after the DOM is fully loaded
document.addEventListener("DOMContentLoaded", () => {
  initializeSpecMonkey();
});

async function initializeSpecMonkey() {
  try {
    // Step 1: Load and parse the config.json
    const config = await loadConfig();

    // Step 2: Get the current page's domain
    const currentDomain = window.location.hostname;

    // Step 3: Check if the domain is in the config
    if (config.domains.includes(currentDomain)) {
      console.log(
        `SpecMonkey: Domain ${currentDomain} is in the config. Proceeding...`
      );

      // Step 4: Fetch the corresponding JSON file from GitHub
      const indexData = await fetchIndexData(currentDomain);

      // Step 5: Process the index data and display boxes
      processIndexData(indexData);
    }
  } catch (error) {
    console.error(`SpecMonkey Error: ${error}`);
  }
}

async function loadConfig() {
  const configURL = browser.runtime.getURL("config.json");
  const response = await fetch(configURL);

  if (!response.ok) {
    throw new Error(`Failed to load config.json: ${response.statusText}`);
  }

  const config = await response.json();

  if (!config.domains || !Array.isArray(config.domains)) {
    throw new Error(
      "Invalid config.json format: 'domains' field is missing or not an array."
    );
  }

  return config;
}

async function fetchIndexData(domain) {
  // Construct the GitHub raw URL for the JSON file
  // Example: https://raw.githubusercontent.com/username/repo/main/example.com.json
  const githubUsername = "jnjaeschke";
  const githubRepo = "specmonkey";
  const githubBranch = "index";

  const jsonURL = `https://raw.githubusercontent.com/${githubUsername}/${githubRepo}/${githubBranch}/${domain}.json`;

  const response = await fetch(jsonURL);

  if (!response.ok) {
    throw new Error(
      `Failed to fetch JSON file for domain ${domain}: ${response.statusText}`
    );
  }

  const jsonData = await response.json();

  if (
    typeof jsonData !== "object" ||
    Array.isArray(jsonData) ||
    jsonData === null
  ) {
    throw new Error(
      `Invalid JSON format for domain ${domain}: Expected a key-value map.`
    );
  }

  return jsonData;
}

function processIndexData(indexData) {
  for (const [fragment, elements] of Object.entries(indexData)) {
    if (Array.isArray(elements)) {
      elements.forEach((element) => {
        if (element.filepath && element.line_number) {
          const anchor =
            document.querySelector(`a[name="${fragment}"], a#${fragment}`) ||
            findAnchorByIdOrName(fragment);

          if (anchor) {
            displayHelloBox(anchor, element);
          } else {
            console.warn(
              `SpecMonkey: Fragment '${fragment}' not found on the page.`
            );
          }
        } else {
          console.warn(
            `SpecMonkey: Element is missing 'filepath' or 'line_number' fields.`
          );
        }
      });
    } else {
      console.warn(
        `SpecMonkey: Expected an array for fragment '${fragment}', but got ${typeof elements}.`
      );
    }
  }
}

function findAnchorByIdOrName(fragment) {
  // Fallback function to locate anchor elements more reliably
  // This can be expanded based on specific requirements
  const elementById = document.getElementById(fragment);
  if (elementById) return elementById;

  const elementsByName = document.getElementsByName(fragment);
  if (elementsByName.length > 0) return elementsByName[0];

  return null;
}

function displayHelloBox(anchor, element) {
  // Create the box container
  const box = document.createElement("div");
  box.style.border = "2px solid red";
  box.style.backgroundColor = "rgba(255, 255, 255, 0.9)";
  box.style.padding = "10px";
  box.style.position = "absolute";
  box.style.zIndex = "1000";
  box.style.borderRadius = "4px";
  box.style.boxShadow = "0 2px 6px rgba(0,0,0,0.2)";
  box.style.maxWidth = "300px";
  box.style.fontFamily = "Arial, sans-serif";
  box.style.fontSize = "14px";

  // Create the content (list of links)
  const content = document.createElement("div");
  content.style.display = "flex";
  content.style.flexDirection = "column";

  // Construct the Searchfox URL
  const searchfoxURL = constructSearchfoxURL(
    element.filepath,
    element.line_number
  );

  // Create the link element
  const link = document.createElement("a");
  link.href = searchfoxURL;
  link.textContent = `View in Searchfox: ${element.filepath}#L${element.line_number}`;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  link.style.color = "#0078D7"; // Searchfox blue color
  link.style.textDecoration = "none";
  link.style.marginBottom = "5px";

  // Append the link to the content
  content.appendChild(link);

  // Optionally, include the "url" field as additional information
  if (element.url) {
    const urlInfo = document.createElement("span");
    urlInfo.textContent = `URL: ${element.url}`;
    urlInfo.style.fontSize = "12px";
    urlInfo.style.color = "#555";
    content.appendChild(urlInfo);
  }

  // Append the content to the box
  box.appendChild(content);

  // Append the box to the body
  document.body.appendChild(box);

  // Position the box near the anchor
  positionBox(box, anchor);

  // Optionally, remove the box after a certain time or add a close button
  setTimeout(() => {
    box.remove();
  }, 10000); // Removes after 10 seconds
}

function constructSearchfoxURL(filepath, lineNumber) {
  // Construct the Searchfox URL based on the filepath and line number

  const baseURL = "https://searchfox.org/"; // Replace with your Searchfox base URL if different
  const repository = "mozilla-central"; // Replace with your repository name
  const filePath = encodeURIComponent(filepath);
  const searchfoxURL = `${baseURL}${repository}/source/${filePath}#L${lineNumber}`;
  return searchfoxURL;
}

function positionBox(box, anchor) {
  const rect = anchor.getBoundingClientRect();
  const scrollY = window.scrollY || window.pageYOffset;
  const scrollX = window.scrollX || window.pageXOffset;

  // Position the box above the anchor element
  box.style.top = `${rect.top + scrollY - box.offsetHeight - 10}px`;
  box.style.left = `${rect.left + scrollX}px`;

  // Ensure the box stays within the viewport
  if (rect.top + scrollY - box.offsetHeight - 10 < scrollY) {
    // Position below if not enough space above
    box.style.top = `${rect.bottom + scrollY + 10}px`;
  }

  if (rect.left + scrollX + box.offsetWidth > scrollX + window.innerWidth) {
    // Adjust left position if box overflows to the right
    box.style.left = `${window.innerWidth - box.offsetWidth - 20}px`;
  }
}
