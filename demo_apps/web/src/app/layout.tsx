import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "PDF Editor Web App",
  description: "Web PDF Editor interface importing WASM package.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="antialiased min-h-screen bg-background text-foreground flex flex-col">
        {children}
      </body>
    </html>
  );
}