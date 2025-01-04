// content.js
initializeSpecMonkey();

/**
 * Checks if the current domain matches any domain in the config.
 * It returns the matched domain or null if no match is found.
 *
 * @param {string} currentDomain - The hostname of the current page.
 * @param {Array<string>} configDomains - The list of domains from config.json.
 * @returns {string|null} - The matched domain or null if no match is found.
 */
function getDomainMatch(currentDomain, configDomains) {
  for (const domain of configDomains) {
    if (currentDomain === domain) {
      return domain; // Exact match
    }
    if (currentDomain.endsWith(`.${domain}`)) {
      return domain; // Subdomain match
    }
  }
  return null; // No match found
}

async function initializeSpecMonkey() {
  try {
    // Step 1: Load and parse the config.json
    const config = await loadConfig();

    // Step 2: Get the current page's domain
    const currentDomain = window.location.hostname;

    // Step 3: Check if the domain is in the config
    const matchingDomain = getDomainMatch(currentDomain, config.domains);
    if (matchingDomain) {
      console.log(
        `SpecMonkey: Domain ${currentDomain} is in the config. Proceeding...`
      );

      // Step 4: Fetch the corresponding JSON file from GitHub
      const indexData = await fetchIndexData(matchingDomain);

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
      try {
        const anchor =
          //   document.querySelector(`a[name="${fragment}"], a#${fragment}`) ||
          findAnchorByIdOrName(fragment);
        if (anchor) {
          displayHelloBox(anchor, elements);
        }
      } catch (e) {
        console.warn(e);
      }
      //   elements.forEach((element) => {
      //     if (element.url && element.filepath && element.line_number) {
      //       const elementhostname = URL.parse(element.url).hostname;
      //       if (window.location.hostname !== elementhostname) {
      //         console.log(
      //           `Hostname not identical for ${element.url}: ${document.hostname} != ${elementhostname}`
      //         );
      //         return;
      //       }
      //     } else {
      //       console.warn(
      //         `SpecMonkey: Element is missing 'filepath' or 'line_number' fields.`
      //       );
      //     }
      //   });
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

function displayHelloBox(anchor, elements) {
  console.log("Building a box");
  // Create the box container
  const box = document.createElement("div");
  box.classList.add("specmonkey-box");
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

  elements.forEach((element) => {
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
    link.textContent = `${element.filepath}#${element.line_number}`;
    link.target = "_blank";
    link.rel = "noopener noreferrer";
    link.style.color = "#0078D7"; // Searchfox blue color
    link.style.textDecoration = "none";
    link.style.marginBottom = "5px";

    // Append the link to the content
    content.appendChild(link);

    // // Optionally, include the "url" field as additional information
    // if (element.url) {
    //   const urlInfo = document.createElement("span");
    //   urlInfo.textContent = `URL: ${element.url}`;
    //   urlInfo.style.fontSize = "12px";
    //   urlInfo.style.color = "#555";
    //   content.appendChild(urlInfo);
    // }

    // Append the content to the box
    box.appendChild(content);
  });
  // Append the box to the body
  document.body.appendChild(box);

  // Position the box near the anchor
  positionBox(box, anchor);
}

function constructSearchfoxURL(filepath, lineNumber) {
  // Construct the Searchfox URL based on the filepath and line number

  const baseURL = "https://searchfox.org/"; // Replace with your Searchfox base URL if different
  const repository = "mozilla-central"; // Replace with your repository name
  //   const filePath = encodeURIComponent(filepath);
  const searchfoxURL = `${baseURL}${repository}/source/${filepath}#${lineNumber}`;
  return searchfoxURL;
}

function positionBox(box, anchor) {
  const viewportWidth = window.innerWidth;
  const boxWidth = box.offsetWidth;

  // Calculate left position: 90% of viewport width minus box width and some padding
  const leftPosition = viewportWidth * 0.9 - boxWidth - 10; // 10px padding from the right edge

  // Calculate top position based on anchor's position
  const rect = anchor.getBoundingClientRect();
  const scrollY = window.scrollY || window.pageYOffset;

  // Set the top position aligned with the anchor's top
  const topPosition = rect.top + scrollY;

  // Apply the calculated positions to the box
  box.style.top = `${topPosition}px`;
  box.style.left = `${leftPosition}px`;
}
