import SwiftUI

struct TodoStatsView: View {
    @ObservedObject var store: TodoStore

    var body: some View {
        VStack(spacing: 16) {
            // Section header
            HStack {
                Label("Progress", systemImage: "chart.bar.fill")
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(.secondary)

                Spacer()

                Text("\(Int(store.completionRate * 100))%")
                    .font(.callout.weight(.bold))
                    .foregroundStyle(store.completionRate > 0.7 ? .green : store.completionRate > 0.3 ? .orange : .red)
            }

            // Stat cards
            HStack(spacing: 12) {
                StatCard(value: store.items.count, label: "Total", icon: "tray.full", color: .indigo)
                StatCard(value: store.activeCount, label: "Active", icon: "circle", color: .orange)
                StatCard(value: store.completedCount, label: "Done", icon: "checkmark.circle.fill", color: .green)
            }

            // Progress track
            VStack(spacing: 6) {
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(Color(.systemGray5))
                        .frame(height: 8)

                    Capsule()
                        .fill(
                            LinearGradient(
                                colors: progressGradient,
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .frame(width: max(8, 200 * store.completionRate), height: 8)
                        .animation(.spring(response: 0.6), value: store.completionRate)
                }

                HStack {
                    Text("\(store.completedCount)/\(store.items.count) tasks")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Spacer()
                }
            }
        }
    }

    private var progressGradient: [Color] {
        if store.completionRate > 0.7 {
            [.mint, .green]
        } else if store.completionRate > 0.3 {
            [.orange, .pink]
        } else {
            [.red, .orange]
        }
    }
}

private struct StatCard: View {
    let value: Int
    let label: String
    let icon: String
    let color: Color

    var body: some View {
        HStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(color.opacity(0.12))
                    .frame(width: 36, height: 36)

                Image(systemName: icon)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(color)
            }

            VStack(alignment: .leading, spacing: 0) {
                Text("\(value)")
                    .font(.title3.weight(.bold))
                    .monospacedDigit()
                    .foregroundStyle(.primary)

                Text(label)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .shadow(color: .black.opacity(0.04), radius: 4, y: 2)
    }
}
