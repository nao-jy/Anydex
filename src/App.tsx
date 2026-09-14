import { QueryClientProvider, QueryClient } from "@tanstack/react-query";
import { MantineProvider } from "@mantine/core";
import { BrowserRouter, Routes, Route } from "react-router";

import '@mantine/core/styles.css';
import { Layout } from "./components/Layout";
import { Index } from "./components/Index";
import { Search } from "./components/Search";

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MantineProvider defaultColorScheme="light" theme={{ primaryColor: "indigo", fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif" }}>
        <AppInner />
      </MantineProvider>
    </QueryClientProvider>
  );
}

function AppInner() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Index />} />
          <Route path="index" element={<Index />} />
          <Route path="search" element={<Search />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
