// content.js
initializeSpecMonkey();
let currentOpenBox = null;
const infoBoxes = new Map();
const infoInlays = new Map();

/**
 * Handles clicks outside the SpecMonkey box to close it.
 *
 * @param {MouseEvent} event - The mouse event triggered by the click.
 */
function handleClickOutsideBox(event) {
  if (currentOpenBox == null) {
    return;
  }
  if (!currentOpenBox.contains(event.target)) {
    currentOpenBox.style.display = "none";
    currentOpenBox = null;
    document.removeEventListener("click", handleClickOutsideBox);
  }
}

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
    const uri = new URL(location.href);

    // Step 3: Check if the domain is in the config
    const matchingDomain = getDomainMatch(uri.hostname, config.domains);
    if (matchingDomain) {
      console.log(
        `SpecMonkey: Domain ${uri.hostname} is in the config. Proceeding...`
      );

      // Step 4: Fetch the corresponding JSON file from searchfox
      const searchfoxData = await fetchSearchfoxData(uri.hostname, config.extensions);

      // Step 5: Convert the searchfox result to index format
      const indexData = convertToIndex(uri, searchfoxData);

      // Step 6: Process the index data and display boxes
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

async function fetchSearchfoxData(domain, extensions) {
  const regex = `pathre:^[^_].*.(${extensions.join('|')})$ re:${domain}/.*#`;
  const jsonURL = `https://searchfox.org/firefox-main/search?q=${encodeURI(regex)}`;
  console.log(`Query: ${jsonURL}`)

  const response = await fetch(jsonURL, {headers: {"Accept": "application/json"}});

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

  if (jsonData["*timedout*"]) {
    console.log(`Query timed out`);
  }
  let entries = (jsonData.normal["Textual Occurrences"] ?? []).concat(jsonData.test["Textual Occurrences"] ?? []);

  console.log(`Got ${entries.length} fragment entries.`);

  return entries
}

function convertToIndex(uri, searchfoxData) {
  const {protocol, hostname, pathname} = uri;
  const domain = hostname;
  const index = new Map();
  const regex = new RegExp(`${domain}\/.*#([\\S]*)[\\s]?`);
  for (const {lines, path: filepath} of searchfoxData) {
    for (const {lno: line_number, line} of lines) {
      const match = regex.exec(line);
      if (!match || match.length < 2) {
        continue;
      }

      const algorithm = match[1];
      const url = `${protocol}//${hostname}/${pathname}#${algorithm}`;
      index.getOrInsert(algorithm, []).push({url, filepath, line_number});
    }
  }
  return index;
}

function processIndexData(indexData) {
  for (const [fragment, elements] of indexData.entries()) {
    if (Array.isArray(elements)) {
      try {
        const anchor =
          //   document.querySelector(`a[name="${fragment}"], a#${fragment}`) ||
          findAnchorByIdOrName(fragment);
        if (anchor) {
          displaySpecmonkeyButton(anchor, elements);
        }
      } catch (e) {
        console.warn(e);
      }
    } else {
      console.warn(
        `SpecMonkey: Expected an array for fragment '${fragment}', but got ${typeof elements}.`
      );
    }
  }
}

function findAnchorByIdOrName(fragment) {
  if (!fragment) {
    return null;
  }
  const elementById = document.getElementById(fragment);
  if (elementById) return elementById;

  const elementsByName = document.getElementsByName(fragment);
  if (elementsByName.length > 0) return elementsByName[0];

  return null;
}

/**
 * Displays the SpecMonkey information box with organized links and a specmonkey button.
 *
 * @param {HTMLElement} anchor - The anchor element near which the box will be displayed.
 * @param {Array<Object>} elements - An array of data elements associated with the anchor.
 */
function displaySpecmonkeyButton(anchor, elements) {
  // Check if a box already exists for this anchor to prevent duplicates
  if (
    anchor.nextSibling &&
    anchor.nextSibling.classList &&
    anchor.nextSibling.classList.contains("specmonkey-box-button")
  ) {
    return;
  }

  // Create the specmonkey button
  const specMonkeyButton = document.createElement("button");
  specMonkeyButton.classList.add("specmonkey-box-button");

  // Create an img element to hold the SVG icon
  const specMonkeyIconFilename = browser.runtime.getURL("searchfox.png"); // Ensure specmonkey.svg is in your extension's directory
  const specMonkeyIcon = document.createElement("img");
  specMonkeyIcon.src = specMonkeyIconFilename;
  specMonkeyIcon.alt = "SpecMonkey"; // Provides accessibility
  specMonkeyIcon.classList.add("specmonkey-box-icon"); // For CSS styling

  // Append the SVG icon to the specmonkey button
  specMonkeyButton.appendChild(specMonkeyIcon);
  // Append the specmonkey button after the anchor
  anchor.insertAdjacentElement(
    anchor.nodeType == "a" ? "afterend" : "beforeend",
    specMonkeyButton
  );

  function createInlayAndBox() {
    if (infoBoxes.has(anchor) && infoInlays.has(anchor)) {
      return infoBoxes.get(anchor), infoInlays.get(anchor);
    }
    // Create the box container
    const infoBox = document.createElement("div");
    infoBox.classList.add("specmonkey-box");
    infoBox.style.display = "none"; // Hidden by default

    // Organize links into categories based on the specified criteria
    const categorizedLinks = categorizeLinks(elements);

    // Create the information box
    const infoInlay = document.createElement("div");
    infoInlay.classList.add("specmonkey-info-box");
    shortCategories = Object.entries(categorizedLinks)
      .filter((el) => el[1].length != 0)
      .map((el) => {
        return `${[...el[0]].reduce(
          (caps, ch) => (ch.match(/[A-Z]/) ? caps + ch : caps),
          ""
        )}: ${el[1].length}`;
      })
      .join(" | ");

    infoInlay.innerText = shortCategories;
    document.body.appendChild(infoInlay);

    const headline = document.createElement("h3");
    const searchfoxQuery = document.createElement("a");
    searchfoxQuery.href = `https://searchfox.org/mozilla-central/search?q=${encodeURIComponent(
      elements[0].url
    )}`;
    searchfoxQuery.target = "_blank";
    searchfoxQuery.rel = "noopener noreferrer";
    searchfoxQuery.textContent = `${elements.length} ${
      elements.length == 1 ? "Reference" : "References"
    } in Gecko (`;
    headline.appendChild(searchfoxQuery);
    infoBox.appendChild(headline);
    // Populate the box with categorized links
    for (const [category, links] of Object.entries(categorizedLinks)) {
      if (links.length === 0) continue; // Skip empty categories

      if (!searchfoxQuery.textContent.endsWith("(")) {
        searchfoxQuery.textContent += ", ";
      }
      searchfoxQuery.textContent += `${links.length} ${category}`;
      // Create and append the headline
      const headline = document.createElement("h3");
      headline.textContent = category;
      infoBox.appendChild(headline);

      // Create and append the links
      links.forEach((linkData) => {
        const link = document.createElement("a");
        link.href = constructSearchfoxURL(
          linkData.filepath,
          linkData.line_number
        );
        link.textContent = `${linkData.filepath}#${linkData.line_number}`;
        link.target = "_blank";
        link.rel = "noopener noreferrer";

        infoBox.appendChild(link);
      });
    }
    searchfoxQuery.textContent += ")";

    // Optional: Add a close button
    const closeButton = document.createElement("span");
    closeButton.textContent = "×";
    closeButton.classList.add("specmonkey-close-button");

    // Append the close button to the box
    infoBox.appendChild(closeButton);

    // Append the box to the body
    document.body.appendChild(infoBox);

    infoBoxes.set(anchor, infoBox);
    infoInlays.set(anchor, infoInlay);

    infoInlay.addEventListener("mouseenter", () => {
      infoInlay.classList.add("visible");
    });

    infoInlay.addEventListener("mouseleave", () => {
      infoInlay.classList.remove("visible");
    });

    // Event listener for the close button
    closeButton.addEventListener("click", () => {
      currentOpenBox.style.display = "none";
      currentOpenBox = null;
    });
  }

  /**
   * Positions the SpecMonkey box immediately adjacent to the specmonkey button.
   */
  function positionBox(box) {
    // Get the bounding rectangle of the specmonkey button
    const buttonRect = specMonkeyButton.getBoundingClientRect();

    // Calculate the position for the box
    const boxTop = buttonRect.bottom + window.scrollY + 5; // 5px below the button
    const boxLeft = buttonRect.left + window.scrollX; // Align with the button's left edge

    // Apply the calculated positions to the box
    box.style.position = "absolute";
    box.style.top = `${boxTop}px`;
    box.style.left = `${boxLeft}px`;

    // Optional: Adjust the box's width based on available space
    const viewportWidth = window.innerWidth;
    const availableWidth = viewportWidth - boxLeft - 20; // 20px padding from the right edge
    if (box.offsetWidth > availableWidth) {
      box.style.width = `${availableWidth}px`;
    }
  }

  specMonkeyButton.addEventListener("mouseenter", () => {
    createInlayAndBox();
    const infoBox = infoBoxes.get(anchor);
    if (currentOpenBox === infoBox) {
      return;
    }
    const infoInlay = infoInlays.get(anchor);
    const buttonRect = specMonkeyButton.getBoundingClientRect();
    infoInlay.style.top = `${buttonRect.top + window.scrollY}px`;
    infoInlay.style.left = `${buttonRect.right + window.scrollX + 10}px`; // 10px gap to the right
    infoInlay.classList.add("visible");
  });

  specMonkeyButton.addEventListener("mouseleave", (event) => {
    const infoInlay = infoInlays.get(anchor);
    if (!infoInlay.contains(event.relatedTarget)) {
      infoInlay.classList.remove("visible");
    }
  });

  // Event listener for clicking the specmonkey button
  specMonkeyButton.addEventListener("click", (event) => {
    event.stopPropagation(); // Prevent the event from bubbling up
    createInlayAndBox();
    const infoBox = infoBoxes.get(anchor);
    const infoInlay = infoInlays.get(anchor);
    infoInlay.classList.remove("visible");
    const isVisible = infoBox.style.display === "block";
    if (!isVisible) {
      if (currentOpenBox && currentOpenBox !== infoBox) {
        currentOpenBox.style.display = "none";
      }
      infoBox.style.display = "block";
      positionBox(infoBox);
      currentOpenBox = infoBox;
      // Add event listener to the document to handle clicks outside
      document.addEventListener("click", handleClickOutsideBox);
    } else {
      infoBox.style.display = "none";
      currentOpenBox = null;
      document.removeEventListener("click", handleClickOutsideBox);
    }
  });
}

/**
 * Categorizes links based on specified criteria.
 *
 * @param {Array<Object>} elements - An array of data elements containing filepath and line_number.
 * @returns {Object} - An object containing categorized links.
 */
function categorizeLinks(elements) {
  const categories = {
    "Web-Platform Test": [],
    Test: [],
    Code: [],
  };

  // Define the file extensions for Mochitest
  const mochitestExtensions = [".html", ".xhtml", ".js"];

  // Iterate through each element and categorize it
  elements.forEach((element) => {
    const filepath = element.filepath;
    const lowerPath = filepath.toLowerCase();
    const extension = `.${lowerPath.split(".").pop()}`;

    if (lowerPath.startsWith("testing/web-platform")) {
      categories["Web-Platform Test"].push(element);
    } else if (
      lowerPath.includes("test") &&
      mochitestExtensions.includes(extension)
    ) {
      categories["Test"].push(element);
    } else {
      categories["Code"].push(element);
    }
  });

  return categories;
}

function constructSearchfoxURL(filepath, lineNumber) {
  // Construct the Searchfox URL based on the filepath and line number

  const baseURL = "https://searchfox.org/"; // Replace with your Searchfox base URL if different
  const repository = "mozilla-central"; // Replace with your repository name
  //   const filePath = encodeURIComponent(filepath);
  const searchfoxURL = `${baseURL}${repository}/source/${filepath}#${lineNumber}`;
  return searchfoxURL;
}
