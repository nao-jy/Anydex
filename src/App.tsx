import { QueryClientProvider, QueryClient } from "@tanstack/react-query";
import { MantineProvider } from "@mantine/core";
import { BrowserRouter, Routes, Route } from "react-router";

import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import "@mantine/notifications/styles.css";
import { Layout } from "./pages/Layout";
import { Index } from "./pages/Index";
import { Mcp } from "./pages/Mcp";
import { Search } from "./pages/Search";
import { Settings } from "./pages/Settings";
import { Tasks } from "./pages/Tasks";
import { OAuth } from "./pages/OAuth";
import { Notifications } from "@mantine/notifications";

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MantineProvider
        defaultColorScheme="light"
        theme={{
          primaryColor: "indigo",
          fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif",
        }}
      >
        <Notifications />
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
          <Route path="oauth" element={<OAuth />} />
          <Route path="mcp" element={<Mcp />} />
          <Route path="tasks" element={<Tasks />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
