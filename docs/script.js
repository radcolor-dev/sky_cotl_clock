const repo = "radcolor/sky_cotl_clock";
const releasesUrl = `https://github.com/${repo}/releases/latest`;
const apiUrl = `https://api.github.com/repos/${repo}/releases/latest`;
const currentVersion = "0.1.4";

const downloadButton = document.querySelector("#downloadButton");
const packageVersion = document.querySelector("#packageVersion");
const cursorLight = document.querySelector(".cursor-light");
const navLinks = [...document.querySelectorAll("nav a")];
const sectionNavLinks = navLinks.filter((link) =>
  link.getAttribute("href")?.startsWith("#"),
);
const sections = sectionNavLinks
  .map((link) => document.querySelector(link.getAttribute("href")))
  .filter(Boolean);

packageVersion.textContent = currentVersion;

function chooseDownloadAsset(assets) {
  const preferredExtensions = [".msi", ".exe", ".zip"];

  return preferredExtensions
    .map((extension) =>
      assets.find((asset) => asset.name.toLowerCase().endsWith(extension)),
    )
    .find(Boolean);
}

async function hydrateLatestRelease() {
  try {
    const response = await fetch(apiUrl, {
      headers: { Accept: "application/vnd.github+json" },
    });

    if (!response.ok) {
      throw new Error(`GitHub responded with ${response.status}`);
    }

    const release = await response.json();
    const asset = chooseDownloadAsset(release.assets || []);
    const version = release.tag_name || `v${currentVersion}`;

    if (asset) {
      downloadButton.href = asset.browser_download_url;
      downloadButton.querySelector("span").textContent = `Download ${version}`;
      return;
    }

    downloadButton.href = release.html_url || releasesUrl;
    downloadButton.querySelector("span").textContent = `View ${version}`;
  } catch (error) {
    downloadButton.href = releasesUrl;
    downloadButton.querySelector("span").textContent = "Download latest version";
  }
}

function setActiveNav() {
  let current;

  sections.forEach((section) => {
    if (section.getBoundingClientRect().top <= 150) {
      current = section;
    }
  });

  sectionNavLinks.forEach((link) => {
    const isActive = current && link.getAttribute("href") === `#${current.id}`;
    link.classList.toggle("active", Boolean(isActive));
  });
}

document.querySelectorAll("[data-reveal]").forEach((element) => {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("revealed");
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.14 },
  );

  observer.observe(element);
});

document.querySelectorAll("details").forEach((details) => {
  details.addEventListener("toggle", () => {
    if (!details.open) {
      return;
    }

    document.querySelectorAll("details").forEach((other) => {
      if (other !== details) {
        other.open = false;
      }
    });
  });
});

document.addEventListener("pointermove", (event) => {
  cursorLight.style.opacity = "1";
  cursorLight.style.transform = `translate(${event.clientX - 180}px, ${event.clientY - 180}px)`;
});

document.addEventListener("pointerleave", () => {
  cursorLight.style.opacity = "0";
});

document.addEventListener("scroll", setActiveNav, { passive: true });

hydrateLatestRelease();
setActiveNav();
