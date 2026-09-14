import { QueryClientProvider, QueryClient } from "@tanstack/react-query";
import { MantineProvider } from "@mantine/core";
import { BrowserRouter, Routes, Route } from "react-router";

import '@mantine/core/styles.css';
import { Layout } from "./components/Layout";
import { Index } from "./components/Index";

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <MantineProvider>
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
          <Route path="index" element={<Index />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
