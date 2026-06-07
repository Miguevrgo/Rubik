import csv
import statistics
import os

def generate_svg_line_chart(k_list, medians, filename):
    width, height = 800, 500
    padding_left, padding_right = 80, 50
    padding_top, padding_bottom = 50, 70
    
    chart_width = width - padding_left - padding_right
    chart_height = height - padding_top - padding_bottom
    
    min_k, max_k = min(k_list), max(k_list)
    min_val, max_val = 0.0, max(medians) * 1.1
    if max_val == 0:
        max_val = 1.0
        
    def get_x(k):
        return padding_left + (k - min_k) / (max_k - min_k) * chart_width
        
    def get_y(val):
        return padding_top + chart_height - (val - min_val) / (max_val - min_val) * chart_height

    svg = []
    svg.append(f'<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">')
    svg.append('<rect width="100%" height="100%" fill="#121824"/>')
    
    for i in range(5):
        val = min_val + i / 4 * (max_val - min_val)
        y = get_y(val)
        svg.append(f'<line x1="{padding_left}" y1="{y}" x2="{width - padding_right}" y2="{y}" stroke="#243049" stroke-width="1" stroke-dasharray="4"/>')
        svg.append(f'<text x="{padding_left - 10}" y="{y + 4}" fill="#94a3b8" font-family="system-ui, sans-serif" font-size="12" text-anchor="end">{val:.2f} ms</text>')
        
    for k in k_list:
        x = get_x(k)
        svg.append(f'<line x1="{x}" y1="{padding_top}" x2="{x}" y2="{padding_top + chart_height}" stroke="#243049" stroke-width="1" stroke-dasharray="4"/>')
        svg.append(f'<text x="{x}" y="{padding_top + chart_height + 25}" fill="#94a3b8" font-family="system-ui, sans-serif" font-size="12" text-anchor="middle">K={k}</text>')

    svg.append(f'<text x="{width / 2}" y="30" fill="#f8fafc" font-family="system-ui, sans-serif" font-size="16" font-weight="bold" text-anchor="middle">Median Solve Time vs. Scramble Depth (K)</text>')
    
    points = []
    for k, val in zip(k_list, medians):
        points.append(f"{get_x(k)},{get_y(val)}")
    points_str = " ".join(points)
    svg.append(f'<polyline points="{points_str}" fill="none" stroke="#3b82f6" stroke-width="3"/>')
    
    for k, val in zip(k_list, medians):
        cx, cy = get_x(k), get_y(val)
        svg.append(f'<circle cx="{cx}" cy="{cy}" r="6" fill="#60a5fa" stroke="#1d4ed8" stroke-width="2"/>')
        svg.append(f'<text x="{cx}" y="{cy - 12}" fill="#f8fafc" font-family="system-ui, sans-serif" font-size="11" font-weight="bold" text-anchor="middle">{val:.2f}</text>')

    svg.append('</svg>')
    with open(filename, 'w') as f:
        f.write("\n".join(svg))

def generate_svg_scatter_plot(k_list, lengths, filename):
    width, height = 800, 500
    padding_left, padding_right = 80, 50
    padding_top, padding_bottom = 50, 70
    
    chart_width = width - padding_left - padding_right
    chart_height = height - padding_top - padding_bottom
    
    min_k, max_k = min(k_list) - 0.5, max(k_list) + 0.5
    min_len, max_len = min(lengths) - 0.5, max(lengths) + 0.5
    
    def get_x(k):
        return padding_left + (k - min_k) / (max_k - min_k) * chart_width
        
    def get_y(val):
        return padding_top + chart_height - (val - min_len) / (max_len - min_len) * chart_height

    svg = []
    svg.append(f'<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">')
    svg.append('<rect width="100%" height="100%" fill="#121824"/>')
    
    for k in range(int(min_k + 0.5), int(max_k + 0.5)):
        x = get_x(k)
        svg.append(f'<line x1="{x}" y1="{padding_top}" x2="{x}" y2="{padding_top + chart_height}" stroke="#243049" stroke-width="1"/>')
        svg.append(f'<text x="{x}" y="{padding_top + chart_height + 25}" fill="#94a3b8" font-family="system-ui, sans-serif" font-size="12" text-anchor="middle">K={k}</text>')
        
    for y_val in range(int(min_len + 0.5), int(max_len + 0.5)):
        y = get_y(y_val)
        svg.append(f'<line x1="{padding_left}" y1="{y}" x2="{width - padding_right}" y2="{y}" stroke="#243049" stroke-width="1"/>')
        svg.append(f'<text x="{padding_left - 10}" y="{y + 4}" fill="#94a3b8" font-family="system-ui, sans-serif" font-size="12" text-anchor="end">{y_val}</text>')

    svg.append(f'<text x="{width / 2}" y="30" fill="#f8fafc" font-family="system-ui, sans-serif" font-size="16" font-weight="bold" text-anchor="middle">Solution Length vs. Scramble Length (Scatter)</text>')
    
    svg.append(f'<line x1="{get_x(min(k_list))}" y1="{get_y(min(k_list))}" x2="{get_x(max(k_list))}" y2="{get_y(max(k_list))}" stroke="#ef4444" stroke-width="2" stroke-dasharray="5"/>')
    
    counts = {}
    for k, length in zip(k_list, lengths):
        counts[(k, length)] = counts.get((k, length), 0) + 1
        
    for (k, length), cnt in counts.items():
        cx = get_x(k)
        cy = get_y(length)
        r = 3 + (cnt ** 0.5) * 3
        opacity = 0.3 + (cnt / max(counts.values())) * 0.5
        svg.append(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="#10b981" fill-opacity="{opacity:.2f}" stroke="#34d399" stroke-width="1.5"/>')
        if cnt > 1:
            svg.append(f'<text x="{cx}" y="{cy + 4}" fill="#ffffff" font-family="system-ui, sans-serif" font-size="10" font-weight="bold" text-anchor="middle">{cnt}</text>')

    svg.append('</svg>')
    with open(filename, 'w') as f:
        f.write("\n".join(svg))

def main():
    csv_file = "training_evaluation_results.csv"
    if not os.path.exists(csv_file):
        print(f"Error: {csv_file} not found.")
        return

    data_by_k = {}
    fewer_steps_cubes = []

    with open(csv_file, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            cube_num = int(row['Cube_Num'])
            k = int(row['K_Scramble'])
            scramble = row['Scramble']
            solution = row['Solution']
            sol_len = int(row['Solution_Length'])
            time_ms = float(row['Time_ms'])

            if k not in data_by_k:
                data_by_k[k] = {
                    'attempts': 0,
                    'solved': 0,
                    'times': [],
                    'lengths': []
                }

            data_by_k[k]['attempts'] += 1
            if sol_len != -1:
                data_by_k[k]['solved'] += 1
                data_by_k[k]['times'].append(time_ms)
                data_by_k[k]['lengths'].append(sol_len)

                if sol_len < k:
                    fewer_steps_cubes.append({
                        'cube_num': cube_num,
                        'k': k,
                        'scramble': scramble,
                        'solution': solution,
                        'sol_len': sol_len,
                        'time_ms': time_ms
                    })

    print("=== SCRAMBLE STATISTICS BY K ===")
    print(f"{'K':<5} | {'Attempts':<10} | {'Solved':<8} | {'Success %':<10} | {'Mean Time (ms)':<15} | {'Median Time (ms)':<17}")
    print("-" * 80)

    k_list = sorted(data_by_k.keys())
    medians = []
    
    all_k = []
    all_lens = []

    for k in k_list:
        stats = data_by_k[k]
        attempts = stats['attempts']
        solved = stats['solved']
        success_rate = (solved / attempts) * 100.0 if attempts > 0 else 0.0
        
        if solved > 0:
            mean_time = sum(stats['times']) / solved
            median_time = statistics.median(stats['times'])
        else:
            mean_time = 0.0
            median_time = 0.0

        medians.append(median_time)
        print(f"{k:<5} | {attempts:<10} | {solved:<8} | {success_rate:<9.2f}% | {mean_time:<14.2f} | {median_time:<16.2f}")
        
        for l in stats['lengths']:
            all_k.append(k)
            all_lens.append(l)

    print("\n=== CUBES SOLVED IN FEWER STEPS THAN K ===")
    print(f"Total: {len(fewer_steps_cubes)} / 800")
    print(f"{'Cube #':<8} | {'K':<3} | {'Sol Len':<7} | {'Time (ms)':<10} | {'Scramble'}")
    print("-" * 100)
    for c in fewer_steps_cubes:
        print(f"{c['cube_num']:<8} | {c['k']:<3} | {c['sol_len']:<7} | {c['time_ms']:<9.2f} | {c['scramble']}")

    generate_svg_line_chart(k_list, medians, "median_time_vs_depth.svg")
    generate_svg_scatter_plot(all_k, all_lens, "solution_vs_scramble_scatter.svg")
    print("\n[Graphics] Generated median_time_vs_depth.svg")
    print("[Graphics] Generated solution_vs_scramble_scatter.svg")

if __name__ == "__main__":
    main()
