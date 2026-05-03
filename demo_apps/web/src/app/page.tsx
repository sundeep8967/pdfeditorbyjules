"use client";

import { useState, useEffect, useRef } from "react";
import { mockSdk } from "@/lib/mock-sdk";
import { Upload, Save, Edit3, Type, Square, Image as ImageIcon, Search, Plus, Minus, Download, Moon, Sun, Sidebar as SidebarIcon, AlignLeft } from "lucide-react";

export default function Home() {
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(0);
  const [pageCount, setPageCount] = useState(0);
  const [pageImage, setPageImage] = useState<string | null>(null);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [darkMode, setDarkMode] = useState(true); // Default to dark as per layout
  const [isEditing, setIsEditing] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Toggle dark mode classes
  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [darkMode]);

  const handleOpenClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setIsLoading(true);
      try {
        const buffer = await file.arrayBuffer();
        await mockSdk.open(buffer);
        setIsOpen(true);
        setPageCount(mockSdk.getPageCount());
        setCurrentPage(0);
        await loadPage(0);
      } catch (error) {
        console.error("Failed to open file", error);
      } finally {
        setIsLoading(false);
      }
    }
  };

  const loadPage = async (index: number) => {
    try {
      const imgData = await mockSdk.renderPage(index);
      setPageImage(imgData);
    } catch (error) {
      console.error("Failed to render page", error);
    }
  };

  const handlePageChange = async (index: number) => {
    if (index >= 0 && index < pageCount) {
      setCurrentPage(index);
      await loadPage(index);
    }
  };

  const handleEditText = async () => {
    if (!isOpen) return;
    setIsEditing(true);
    setIsLoading(true);
    try {
      // Mock interaction for editing text
      const searchStr = prompt("Enter text to replace:") || "Old Text";
      const replaceStr = prompt("Enter replacement text:") || "New Text";

      await mockSdk.replaceText(searchStr, replaceStr);
      // Reload current page to see mock changes
      await loadPage(currentPage);
    } catch (error) {
      console.error("Failed to edit text", error);
    } finally {
      setIsLoading(false);
      setIsEditing(false);
    }
  };

  const handleSave = async () => {
    if (!isOpen) return;
    setIsLoading(true);
    try {
      await mockSdk.save();
      alert("Mock document saved successfully!");
    } catch (error) {
      console.error("Failed to save document", error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex flex-col h-screen overflow-hidden bg-background text-foreground transition-colors duration-200">
      {/* Top App Bar */}
      <header className="h-14 border-b border-border flex items-center justify-between px-4 bg-card shrink-0 z-10 shadow-sm">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="p-2 rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
            title="Toggle Sidebar"
          >
            <SidebarIcon className="w-5 h-5" />
          </button>
          <div className="w-8 h-8 bg-primary rounded flex items-center justify-center text-primary-foreground font-bold ml-2">
            P
          </div>
          <span className="font-semibold text-lg ml-2 hidden sm:block">PDF Editor SDK</span>
        </div>

        {/* Toolbar Center Actions */}
        <div className="flex items-center gap-1 md:gap-2">
          {!isOpen ? (
            <button
              onClick={handleOpenClick}
              disabled={isLoading}
              className="flex items-center gap-2 bg-primary text-primary-foreground px-4 py-2 rounded-md hover:bg-primary/90 transition-colors text-sm font-medium disabled:opacity-50"
            >
              <Upload className="w-4 h-4" />
              Open PDF
            </button>
          ) : (
            <>
              <div className="flex items-center border border-border rounded-md bg-background overflow-hidden mr-2">
                <button
                  onClick={handleEditText}
                  disabled={isLoading}
                  className={`p-2 hover:bg-accent transition-colors ${isEditing ? 'bg-accent' : ''}`}
                  title="Edit Text"
                >
                  <Type className="w-4 h-4" />
                </button>
                <button className="p-2 hover:bg-accent transition-colors border-l border-border" title="Add Shape">
                  <Square className="w-4 h-4" />
                </button>
                <button className="p-2 hover:bg-accent transition-colors border-l border-border" title="Add Image">
                  <ImageIcon className="w-4 h-4" />
                </button>
              </div>

              <div className="hidden md:flex items-center px-3 gap-3 border-r border-border h-8 mr-2">
                <button className="hover:text-primary transition-colors"><Minus className="w-4 h-4" /></button>
                <span className="text-sm text-muted-foreground w-12 text-center">100%</span>
                <button className="hover:text-primary transition-colors"><Plus className="w-4 h-4" /></button>
              </div>

              <button
                onClick={handleSave}
                disabled={isLoading}
                className="flex items-center gap-2 bg-primary text-primary-foreground px-4 py-2 rounded-md hover:bg-primary/90 transition-colors text-sm font-medium disabled:opacity-50"
              >
                <Save className="w-4 h-4" />
                <span className="hidden sm:inline">Save</span>
              </button>
              <button className="p-2 text-muted-foreground hover:text-foreground transition-colors ml-1">
                <Download className="w-5 h-5" />
              </button>
            </>
          )}
          <input
            type="file"
            ref={fileInputRef}
            className="hidden"
            accept=".pdf"
            onChange={handleFileChange}
          />
        </div>

        {/* Right Actions */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setDarkMode(!darkMode)}
            className="p-2 rounded-md hover:bg-accent transition-colors"
          >
            {darkMode ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
          </button>
          <div className="w-8 h-8 rounded-full bg-secondary border border-border overflow-hidden ml-2 hidden sm:block">
            <img src="https://api.dicebear.com/7.x/avataaars/svg?seed=Felix" alt="User" className="w-full h-full" />
          </div>
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden relative">
        {/* Left Navigation Rail / Drawer */}
        <aside
          className={`absolute md:relative z-20 flex-shrink-0 w-64 h-full bg-card border-r border-border transition-all duration-300 ease-in-out ${
            isSidebarOpen ? 'translate-x-0' : '-translate-x-full md:w-0 md:border-none'
          }`}
        >
          <div className="p-4 border-b border-border flex items-center gap-2">
            <AlignLeft className="w-4 h-4 text-muted-foreground" />
            <h2 className="text-sm font-medium uppercase tracking-wider text-muted-foreground">Thumbnails</h2>
          </div>

          <div className="p-4 overflow-y-auto h-[calc(100%-3rem)] space-y-4">
            {isOpen ? (
              Array.from({ length: pageCount }).map((_, i) => (
                <div
                  key={i}
                  onClick={() => handlePageChange(i)}
                  className={`group cursor-pointer rounded-lg border-2 p-1 transition-all ${
                    currentPage === i ? 'border-primary bg-primary/5' : 'border-transparent hover:border-border hover:bg-accent'
                  }`}
                >
                  <div className="bg-background border border-border rounded aspect-[1/1.4] w-full flex items-center justify-center relative overflow-hidden shadow-sm">
                    <span className="text-4xl text-muted-foreground/30 font-bold">{i + 1}</span>
                  </div>
                  <div className="text-center text-xs mt-2 text-muted-foreground font-medium">Page {i + 1}</div>
                </div>
              ))
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-muted-foreground opacity-50 space-y-4">
                <Search className="w-8 h-8" />
                <p className="text-sm text-center">Open a PDF to view thumbnails</p>
              </div>
            )}
          </div>
        </aside>

        {/* Central Canvas Workspace */}
        <main className="flex-1 bg-secondary/30 relative overflow-auto flex flex-col items-center p-4 sm:p-8">
          {isLoading && (
            <div className="absolute inset-0 bg-background/50 backdrop-blur-sm z-30 flex items-center justify-center">
              <div className="flex flex-col items-center gap-4">
                <div className="w-10 h-10 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
                <p className="text-primary font-medium animate-pulse">Processing SDK Call...</p>
              </div>
            </div>
          )}

          {isOpen && pageImage ? (
            <div className="w-full max-w-4xl bg-white shadow-xl rounded-sm transition-all duration-300 relative group min-h-[800px] flex items-center justify-center overflow-hidden border border-gray-200">
               {/* Use an img tag for the SVG data URI */}
               <img src={pageImage} alt={`Page ${currentPage + 1}`} className="w-full h-auto object-contain" />

               {/* Mock selection overlay on hover */}
               <div className="absolute inset-0 border-2 border-primary/0 group-hover:border-primary/20 pointer-events-none transition-colors"></div>
            </div>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center text-center max-w-md mx-auto">
              <div className="w-20 h-20 bg-card rounded-2xl flex items-center justify-center mb-6 shadow-sm border border-border">
                <Upload className="w-10 h-10 text-primary" />
              </div>
              <h1 className="text-2xl font-bold mb-2">Welcome to PDF Editor SDK</h1>
              <p className="text-muted-foreground mb-8">
                A blazingly fast, modern web interface powered by our WebAssembly PDF engine.
              </p>
              <button
                onClick={handleOpenClick}
                className="bg-primary text-primary-foreground px-6 py-3 rounded-lg hover:bg-primary/90 transition-colors font-medium shadow-md flex items-center gap-2"
              >
                <Upload className="w-5 h-5" />
                Upload Document
              </button>
            </div>
          )}

          {/* Bottom Floating Toolbar (optional) */}
          {isOpen && (
            <div className="fixed bottom-6 left-1/2 -translate-x-1/2 bg-card border border-border shadow-lg rounded-full px-4 py-2 flex items-center gap-4 z-20">
              <button
                onClick={() => handlePageChange(currentPage - 1)}
                disabled={currentPage === 0}
                className="p-1.5 rounded-full hover:bg-accent disabled:opacity-30 transition-colors"
              >
                <Minus className="w-4 h-4" />
              </button>
              <span className="text-sm font-medium w-20 text-center">
                {currentPage + 1} / {pageCount}
              </span>
              <button
                onClick={() => handlePageChange(currentPage + 1)}
                disabled={currentPage === pageCount - 1}
                className="p-1.5 rounded-full hover:bg-accent disabled:opacity-30 transition-colors"
              >
                <Plus className="w-4 h-4" />
              </button>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}