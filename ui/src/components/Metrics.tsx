import { BarChart } from "@mui/x-charts/BarChart";
import { useLoaderData } from "react-router";
import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
type SpanLength = {
  index: number;
  range: string;
  count: number;
};
type SpanTools = {
  cnt_spns_with_tools: number;
  cnt_spns_without_tools: number;
  date: Date;
};
type TelemetryMetrics = {
  queryLatency: readonly SpanLength[];
  ingestionLatency: readonly SpanTools[];
  queryTools: readonly SpanTools[];
};
const centerStyle = { display: "flex", justifyContent: "center" };
const Metrics = () => {
  const { queryLatency, ingestionLatency, queryTools } =
    useLoaderData() as TelemetryMetrics;
  return (
    <>
      <Grid size={{ xs: 12 }}>
        <Typography style={centerStyle}>Tool use</Typography>
        <BarChart
          dataset={queryTools}
          height={300}
          xAxis={[{ dataKey: "date" }]}
          series={[
            {
              dataKey: "cnt_spns_with_tools",
              label: "Tool invocations",
            },
            {
              dataKey: "cnt_spns_without_tools",
              label: "Without tool invocations",
            },
          ]}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6 }}>
        <Typography style={centerStyle}>Query response time</Typography>
        <BarChart
          dataset={queryLatency}
          height={300}
          xAxis={[{ dataKey: "range" }]}
          series={[
            {
              dataKey: "count",
            },
          ]}
        />
      </Grid>
      <Grid size={{ xs: 12, md: 6 }}>
        <Typography style={centerStyle}>Ingestion response time</Typography>
        <BarChart
          dataset={ingestionLatency}
          height={300}
          xAxis={[{ dataKey: "range" }]}
          series={[
            {
              dataKey: "count",
            },
          ]}
        />
      </Grid>
    </>
  );
};
export default Metrics;
