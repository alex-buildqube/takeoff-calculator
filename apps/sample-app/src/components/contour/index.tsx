import { ContourWrapper } from "local-bindings";
import { useEffect, useRef } from "react";
import Plot from "react-plotly.js";
import { TEST_CONTOUR } from "./test-data";

const contour = new ContourWrapper(TEST_CONTOUR);

export const ContourComponent = () => {
	const ref = useRef<HTMLCanvasElement>(null);
	const data = contour
		.getScatterData(5)
		?.reduce<{ x: number[]; y: number[]; z: number[] }>(
			(acc, cur) => {
				acc.x.push(cur.x);
				acc.y.push(cur.y);
				acc.z.push(cur.z);
				return acc;
			},
			{ x: [], y: [], z: [] },
		);
	const minZ = Math.min(...(data?.z ?? []));
	const maxZ = Math.max(...(data?.z ?? []));

	const pointData = contour
		.getSurfacePoints()
		?.reduce<{ x: number[]; y: number[]; z: number[] }>(
			(acc, cur) => {
				acc.x.push(cur.x);
				acc.y.push(cur.y);
				acc.z.push(cur.z);
				return acc;
			},
			{ x: [], y: [], z: [] },
		);

	useEffect(() => {
		if (ref.current && data?.x?.length && data?.y?.length) {
			const canvas = ref.current;
			canvas.width = Math.max(...data.x) / 2;
			canvas.height = Math.max(...data.y) / 2;
			const context = canvas.getContext("2d");
			if (context) {
				context.clearRect(0, 0, canvas.width, canvas.height);
				for (const line of TEST_CONTOUR.lines) {
					line.points.forEach((point, idx) => {
						if (idx > 0) {
							context.beginPath();
							context.moveTo(
								line.points[idx - 1].x / 2,
								line.points[idx - 1].y / 2,
							);
							context.lineTo(point.x / 2, point.y / 2);
							// Change color based on elevation
							// Min should be blue max should be red
							const min = minZ;
							const max = maxZ;
							const value = line.elevation;
							const ratio = max > min ? (value - min) / (max - min) : 0.5;
							const blue = Math.round(255 * (1 - ratio));
							const red = Math.round(255 * ratio);
							context.strokeStyle = `rgba(${red}, 0, ${blue}, 1)`;
							// context.strokeStyle = `rgba(255, 0, 0, ${line.elevation / 500})`;

							context.stroke();
						}
					});
				}
			}
		}
	}, []);
	return (
		<div>
			<div style={{ width: "100%", height: "100%" }}>
				<h1>Contour</h1>

				<div
					style={{
						width: "100%",
						height: "100%",
						display: "flex",
						flexDirection: "row",
						gap: 10,
						justifyContent: "space-between",
					}}
				>
					<canvas
						style={{
							margin: 50,
							flex: 1,
							transform: "rotateX(180deg)",
						}}
						ref={ref}
					/>
					<Plot
						data={[
							{
								x: data?.x,
								y: data?.y,
								z: data?.z,
								type: "scatter",
								mode: "markers",
								marker: {
									size: 10,

									color: data?.z,
									colorscale: "Portland",
								},
							},
						]}
						layout={{
							// width: 640,
							// height: 480,
							width: 500,
							height: 500,
							autosize: true,
							title: {
								text: "Contour",
							},
							scene: {
								zaxis: {
									range: [minZ - maxZ / 2, maxZ * 2],
								},
								// xaxis: {
								//     range: [0, 1200]
								// },
								// yaxis: {
								//     range: [0, 1200]
								// }
							},
						}}
						config={{
							responsive: true,
						}}
						useResizeHandler={true} // Crucial prop for automatic resizing
						style={{ flex: 1 }}
					/>
				</div>
				<Plot
					data={[
						{
							...pointData,
							type: "scatter3d",
							mode: "markers",
							marker: {
								size: 5,
								// color: 'blue',
								color: pointData?.z,
								colorscale: "Portland",
							},
							// zmin: 0,
							// zmax: 5000
						},
					]}
					layout={{
						width: 1000,
						height: 600,

						autosize: true,
						title: {
							text: "Contour Points",
						},
						scene: {
							zaxis: {
								range: [minZ - maxZ / 2, maxZ * 2],
							},
						},
					}}
					config={{
						responsive: true,
					}}
					useResizeHandler={true} // Crucial prop for automatic resizing
					style={{ width: "100%", height: "100%" }}
				/>
				<Plot
					data={[
						{
							...data,
							type: "scatter3d",
							mode: "markers",
							marker: {
								size: 2,
								// color: 'blue',
								color: data?.z,
								colorscale: "Portland",
							},
							// zmin: 0,
							// zmax: 5000
						},
					]}
					layout={{
						// width: 640,
						// height: 480,
						width: 1000,
						height: 1000,
						autosize: true,
						title: {
							text: "Contour",
						},
						scene: {
							zaxis: {
								range: [minZ - maxZ / 2, maxZ * 2],
							},
							// xaxis: {
							//     range: [0, 1200]
							// },
							// yaxis: {
							//     range: [0, 1200]
							// }
						},
					}}
					config={{
						responsive: true,
					}}
					useResizeHandler={true} // Crucial prop for automatic resizing
					style={{ width: "100%", height: "100%" }}
				/>
			</div>
		</div>
	);
};
