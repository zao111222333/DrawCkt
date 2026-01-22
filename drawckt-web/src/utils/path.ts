/**
 * Get the base path for the application (for subfolder deployment)
 * Returns the base path without trailing slash, e.g., '/path/to/drawckt' or ''
 */
export function getBasePath(): string {
  // Get base path from current location
  const path = window.location.pathname;
  
  // Remove index.html if present
  let basePath = path.replace(/\/index\.html$/, '');
  
  // If we're at root, return empty string
  if (basePath === '' || basePath === '/') {
    return '';
  }
  
  // Find the directory containing index.html
  // For /path/to/drawckt/index.html, basePath should be /path/to/drawckt
  // For /path/to/drawckt/, basePath should be /path/to/drawckt
  basePath = basePath.replace(/\/$/, '');
  
  // If basePath is empty after removing trailing slash, we're at root
  if (basePath === '') {
    return '';
  }
  
  return basePath;
}

/**
 * Convert an absolute path to a relative path based on base path
 */
export function toRelativePath(absolutePath: string): string {
  const basePath = getBasePath();
  if (!basePath) {
    return absolutePath;
  }
  
  // Remove base path prefix
  if (absolutePath.startsWith(basePath)) {
    return absolutePath.slice(basePath.length);
  }
  
  return absolutePath;
}

/**
 * Convert a relative path to an absolute path based on base path
 */
export function toAbsolutePath(relativePath: string): string {
  const basePath = getBasePath();
  // Remove leading slash from relative path
  const cleanPath = relativePath.replace(/^\/+/, '');
  
  if (!basePath) {
    return '/' + cleanPath;
  }
  
  return basePath + '/' + cleanPath;
}

/**
 * Get embedded file path (for WASM routing)
 * Always returns absolute path starting with /embedded/
 */
export function getEmbeddedPath(relativePath: string): string {
  // Remove leading slash and base path
  const cleanPath = relativePath.replace(/^\/+/, '').replace(/^embedded\//, '');
  const basePath = getBasePath();
  
  if (!basePath) {
    return '/embedded/' + cleanPath;
  }
  
  return basePath + '/embedded/' + cleanPath;
}

