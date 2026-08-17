import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "KagoRoute — USSD & SMS Integration Layer",
  description:
    "Mirror your web and smartphone logic onto feature phones via USSD and SMS. One schema, every phone.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-[#060a09] text-zinc-200 antialiased">
        {children}
      </body>
    </html>
  );
}
