export class PdfDocumentMock {
  private documentId: string | null = null;
  private isModified: boolean = false;

  async open(fileBuffer: ArrayBuffer): Promise<boolean> {
    console.log("PdfDocumentMock: Opening PDF document of size", fileBuffer.byteLength);
    // Simulate loading time
    await new Promise(resolve => setTimeout(resolve, 800));
    this.documentId = "mock-doc-123";
    this.isModified = false;
    return true;
  }

  async renderPage(pageIndex: number): Promise<string> {
    console.log(`PdfDocumentMock: Rendering page ${pageIndex}`);
    if (!this.documentId) {
      throw new Error("No document opened");
    }
    // Return a base64 encoded transparent 1x1 png or simple svg placeholder for the mock
    // Here we'll return a data URI for an SVG showing it's a rendered page
    const svg = `
      <svg width="800" height="1100" xmlns="http://www.w3.org/2000/svg">
        <rect width="100%" height="100%" fill="white" />
        <rect width="100%" height="100%" fill="none" stroke="#e5e7eb" stroke-width="4"/>
        <text x="50%" y="50%" font-family="Arial" font-size="40" fill="#9ca3af" text-anchor="middle" dominant-baseline="middle">
          Mock PDF Page ${pageIndex + 1}
        </text>
        ${this.isModified ? `<text x="50%" y="60%" font-family="Arial" font-size="30" fill="#3b82f6" text-anchor="middle" dominant-baseline="middle">(Modified)</text>` : ''}
      </svg>
    `;
    return `data:image/svg+xml;base64,${btoa(svg)}`;
  }

  async replaceText(searchString: string, replacementString: string): Promise<number> {
    console.log(`PdfDocumentMock: Replacing "${searchString}" with "${replacementString}"`);
    if (!this.documentId) {
      throw new Error("No document opened");
    }
    await new Promise(resolve => setTimeout(resolve, 500));
    this.isModified = true;
    // Return number of replacements
    return 1;
  }

  async save(): Promise<ArrayBuffer> {
    console.log("PdfDocumentMock: Saving PDF document");
    if (!this.documentId) {
      throw new Error("No document opened");
    }
    await new Promise(resolve => setTimeout(resolve, 1000));
    this.isModified = false;
    // Return a dummy ArrayBuffer
    return new ArrayBuffer(1024);
  }

  getPageCount(): number {
    return this.documentId ? 5 : 0;
  }
}

// Singleton instance to be used across the app
export const mockSdk = new PdfDocumentMock();
