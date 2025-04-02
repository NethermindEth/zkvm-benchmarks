import polars as pl
import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns # Import seaborn
import os # Import os for path joining

def add_value_labels(ax, fmt='.2e', conversion_factor=1, unit_suffix=''):
    """Add value labels to the bars in the barplot, with optional conversion and suffix.

    Args:
        ax: The matplotlib axes containing the barplot.
        fmt: Format string for the value labels.
        conversion_factor: Factor to multiply the value by before formatting (e.g., 1/60 for sec to min).
        unit_suffix: String to append after the formatted value (e.g., ' min').
    """
    # For each bar in the plot
    for p in ax.patches:
        height = p.get_height()

        if np.isfinite(height) and height > 0:
            # Apply conversion factor
            value_to_display = height * conversion_factor
            # Format text with specified format and add suffix
            text = f'{value_to_display:{fmt}}{unit_suffix}'

            ax.annotate(
                text,
                (p.get_x() + p.get_width() / 2, height), # Annotation placed at original height
                ha='center',
                va='bottom',
                fontsize=8,
                rotation=45,
                xytext=(0, 5),
                textcoords='offset points'
            )

def _plot_metric_on_ax(ax, df, y_column, y_label, title, use_log_scale):
    """Helper function to draw the grouped bar chart on a given Axes object."""
    sns.barplot(
        data=df,
        x="program",
        y=y_column,
        hue="prover",
        ax=ax
    )

    # Determine formatting for value labels
    if y_column == 'cycles':
        fmt = '.2e'
        conversion = 1
        suffix = ''
        unit = ''
    else:
        fmt = '.2f'
        conversion = 1 / 60
        suffix = ' min'
        unit = '' # Unit is part of y_label now

    add_value_labels(ax, fmt=fmt, conversion_factor=conversion, unit_suffix=suffix)

    ax.set_xlabel("Program")
    y_axis_label = f"{y_label}{unit}"
    if use_log_scale:
        y_axis_label += " (Log Scale)"
    ax.set_ylabel(y_axis_label)
    ax.set_title(title)
    ax.tick_params(axis='x', rotation=45)

    if use_log_scale:
        ax.set_yscale('log')
    
    # Adjust legend position slightly if needed (can be customized)
    ax.legend(loc='upper right', fontsize='small')

def create_grouped_bar_chart(df, y_column, y_label, title_metric, filename, use_log_scale=False):
    """Create a single grouped bar chart and save it to a file."""
    fig, ax = plt.subplots(figsize=(14, 9))
    
    # Use the helper to draw on the axes
    _plot_metric_on_ax(ax, df, y_column, y_label, title_metric, use_log_scale)

    plt.tight_layout()
    
    output_dir = "plots"
    os.makedirs(output_dir, exist_ok=True)
    full_path = os.path.join(output_dir, filename)
    
    plt.savefig(full_path, bbox_inches='tight', dpi=150)
    print(f"Saved plot to {full_path}")
    plt.close(fig)

def create_combined_duration_plot(df, filename):
    """Creates a 2x2 grid of duration plots and saves to a single file."""
    fig, axes = plt.subplots(2, 2, figsize=(18, 12), sharey=False) # Adjust figsize, sharey=False if scales differ
    fig.suptitle("Benchmark Durations for RETH Programs", fontsize=16)
    
    metrics_to_plot = [
        {"ax": axes[0, 0], "y_column": "core_prove_duration", "title": "Core Proof"},
        {"ax": axes[0, 1], "y_column": "prove_duration", "title": "Core + Recursive Proofs"},
        {"ax": axes[1, 0], "y_column": "execution_duration", "title": "Execution Trace"},
        {"ax": axes[1, 1], "y_column": "compress_prove_duration", "title": "Recursive Proof"},
    ]

    for plot_info in metrics_to_plot:
        _plot_metric_on_ax(
            ax=plot_info["ax"],
            df=df,
            y_column=plot_info["y_column"],
            y_label="Minutes",
            title=plot_info["title"],
            use_log_scale=False # Linear scale for durations
        )

    plt.tight_layout(rect=[0, 0.03, 1, 0.95]) # Adjust rect for suptitle

    output_dir = "plots"
    os.makedirs(output_dir, exist_ok=True)
    full_path = os.path.join(output_dir, filename)

    plt.savefig(full_path, bbox_inches='tight', dpi=150)
    print(f"Saved combined plot to {full_path}")
    plt.close(fig)

def main():
    sns.set_theme(style="whitegrid")

    try:
        df_raw = pl.read_csv(
            "../results/benchmark_latest.csv",
            schema_overrides={'cycles': pl.Int64}
        )
    except FileNotFoundError:
        print("Error: Could not find '../results/benchmark_latest.csv'.")
        return
    except Exception as e:
        print(f"Error reading CSV: {e}")
        return

    # --- Filter Data --- 
    cols_to_check = ["cycles", "prove_duration", "execution_duration", "core_prove_duration", "compress_prove_duration"]
    df_all_valid = df_raw.filter(pl.all_horizontal(pl.col(c).is_not_null() for c in cols_to_check))

    if df_all_valid.height < df_raw.height:
        print(f"Warning: Removed {df_raw.height - df_all_valid.height} rows with null values in key columns.")

    if df_all_valid.height == 0:
        print("Error: No valid data left after initial null filtering. Cannot generate plots.")
        return

    # --- Generate and Save Plots --- 
    # 1. Cycles plot (All Programs, Log Scale)
    print("Generating Cycles plot for all programs (Log Scale)...")
    create_grouped_bar_chart(
        df_all_valid, 
        y_column="cycles", 
        y_label="Cycles", 
        title_metric="Cycles", 
        filename="cycles_all_programs_log.png", 
        use_log_scale=True
    )

    # 2. Filter further for duration plots (RETH programs only)
    target_programs = ["reth_19409768", "reth_17106222"]
    df_reth_valid = df_all_valid.filter(pl.col("program").is_in(target_programs))

    # 3. Create Combined Duration Plot (RETH programs only, Linear Scale)
    if df_reth_valid.height > 0:
        print(f"Generating Combined Duration plot for programs: {target_programs} (Linear Scale)...")
        create_combined_duration_plot(df_reth_valid, filename="combined_durations_reth_only.png")
    else:
        print(f"Warning: No valid data found for the target programs ({target_programs}) after filtering. Skipping combined duration plot.")

    print("Finished saving plots.")
    # Remove final plt.show() as plots are saved directly
    # plt.show()

if __name__ == "__main__":
    main()
