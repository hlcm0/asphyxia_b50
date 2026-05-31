import { toPng } from "html-to-image";

export async function renderBoardPng(board: HTMLElement) {
  const exportBoard = board.cloneNode(true) as HTMLElement;
  exportBoard.id = "export-board-fixed";
  exportBoard.classList.add("export-fixed");

  const exportHost = document.createElement("div");
  exportHost.className = "fixed-export-host";
  exportHost.appendChild(exportBoard);
  document.body.appendChild(exportHost);

  try {
    await waitForImages(exportBoard);
    await waitForBackgroundImages(exportBoard);
    const dataUrl = await toPng(exportBoard, {
      cacheBust: false,
      pixelRatio: 2,
      backgroundColor: "#f8f9ff",
      width: exportBoard.scrollWidth,
      height: exportBoard.scrollHeight
    });
    return dataUrlToBytes(dataUrl);
  } finally {
    exportHost.remove();
  }
}

export function sanitizeName(value: string) {
  return value.replace(/[^a-z0-9]+/gi, "_").replace(/^_+|_+$/g, "") || "player";
}

async function waitForImages(root: HTMLElement) {
  const images = Array.from(root.querySelectorAll("img"));
  await Promise.all(
    images.map(
      (image) =>
        new Promise<void>((resolve, reject) => {
          if (image.complete && image.naturalWidth > 0) {
            resolve();
            return;
          }
          image.onload = () => resolve();
          image.onerror = () => reject(new Error(`Failed to load ${image.alt || "image"}`));
        })
    )
  );
}

async function waitForBackgroundImages(root: HTMLElement) {
  const urls = new Set<string>();
  [root, ...Array.from(root.querySelectorAll<HTMLElement>("*"))].forEach((element) => {
    extractCssUrls(getComputedStyle(element).backgroundImage).forEach((url) => urls.add(url));
  });

  await Promise.all(Array.from(urls).map(loadImage));
}

function extractCssUrls(value: string) {
  const urls: string[] = [];
  const pattern = /url\((?:"([^"]+)"|'([^']+)'|([^)]*))\)/g;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(value)) !== null) {
    const url = (match[1] || match[2] || match[3] || "").trim();
    if (url && url !== "none") {
      urls.push(url);
    }
  }

  return urls;
}

function loadImage(url: string) {
  return new Promise<void>((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve();
    image.onerror = () => reject(new Error(`Failed to load background image: ${url}`));
    image.src = url;
  });
}

function dataUrlToBytes(dataUrl: string) {
  const base64 = dataUrl.split(",")[1] ?? "";
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
