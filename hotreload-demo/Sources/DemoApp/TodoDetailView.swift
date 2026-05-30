import SwiftUI

struct TodoDetailView: View {
    @ObservedObject var store: TodoStore
    let item: TodoItem

    @Environment(\.dismiss) private var dismiss
    @State private var animateContent = false

    private var isCompleted: Bool {
        store.items.first(where: { $0.id == item.id })?.isCompleted ?? item.isCompleted
    }

    private var priorityColor: Color { item.priority.color }

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // ── Hero Header ──
                heroHeader

                // ── Content Cards ──
                VStack(spacing: 16) {
                    metaRow
                    quickStats
                    if !item.notes.isEmpty { notesSection }
                    actionButtons
                }
                .padding(.horizontal, 20)
                .padding(.top, -24)
                .offset(y: animateContent ? 0 : 40)
                .opacity(animateContent ? 1 : 0)
            }
            .padding(.bottom, 32)
        }
        .background(Color(.systemGroupedBackground))
        .ignoresSafeArea(edges: .top)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarBackground(.hidden, for: .navigationBar)
        .onAppear {
            withAnimation(.spring(response: 0.6, dampingFraction: 0.8).delay(0.1)) {
                animateContent = true
            }
        }
    }

    // MARK: - Hero Header

    private var heroHeader: some View {
        ZStack(alignment: .bottom) {
            // Background gradient
            LinearGradient(
                colors: [
                    priorityColor.opacity(0.6),
                    priorityColor.opacity(0.2),
                    Color(.systemGroupedBackground)
                ],
                startPoint: .top,
                endPoint: .bottom
            )
            .frame(height: 260)

            VStack(spacing: 16) {
                // Status ring
                ZStack {
                    Circle()
                        .stroke(isCompleted ? Color.green.opacity(0.3) : Color.gray.opacity(0.2), lineWidth: 4)
                        .frame(width: 72, height: 72)

                    Circle()
                        .trim(from: 0, to: isCompleted ? 1 : 0)
                        .stroke(isCompleted ? Color.green : priorityColor, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                        .frame(width: 72, height: 72)
                        .rotationEffect(.degrees(-90))

                    Image(systemName: isCompleted ? "checkmark" : item.priority.icon)
                        .font(.title.weight(.semibold))
                        .foregroundStyle(isCompleted ? .green : priorityColor)
                }

                // Title
                Text(item.title)
                    .font(.system(size: 26, weight: .bold, design: .rounded))
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 24)

                // Meta badges
                HStack(spacing: 8) {
                    badge(icon: item.priority.icon, text: item.priority.label, color: priorityColor)
                    badge(icon: "tag", text: item.category, color: .purple)
                    badge(
                        icon: isCompleted ? "checkmark.circle" : "clock",
                        text: isCompleted ? "Done" : "Active",
                        color: isCompleted ? .green : .orange
                    )
                }
            }
            .padding(.bottom, 32)
        }
    }

    // MARK: - Meta Row

    private var metaRow: some View {
        HStack(spacing: 12) {
            metaChip(
                icon: "calendar",
                label: "Created",
                value: item.createdAt.formatted(date: .abbreviated, time: .shortened)
            )

            metaChip(
                icon: "clock.arrow.circlepath",
                label: "Age",
                value: item.createdAt.formatted(.relative(presentation: .named))
            )
        }
    }

    // MARK: - Quick Stats

    private var quickStats: some View {
        HStack(spacing: 12) {
            statCard(
                icon: "textformat.abc",
                label: "Title Length",
                value: "\(item.title.count) chars",
                color: .blue
            )

            statCard(
                icon: "note.text",
                label: "Notes",
                value: item.notes.isEmpty ? "None" : "\(item.notes.count) chars",
                color: .indigo
            )

            statCard(
                icon: "number",
                label: "ID",
                value: String(item.id.uuidString.prefix(8)),
                color: .gray
            )
        }
    }

    // MARK: - Notes Section

    private var notesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("Notes", systemImage: "note.text")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)

            Text(item.notes)
                .font(.body)
                .lineSpacing(4)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(16)
                .background(Color(.systemBackground))
                .clipShape(RoundedRectangle(cornerRadius: 16))
                .shadow(color: .black.opacity(0.04), radius: 8, y: 2)
        }
        .padding(16)
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 20))
        .shadow(color: .black.opacity(0.06), radius: 12, y: 4)
    }

    // MARK: - Action Buttons

    private var actionButtons: some View {
        VStack(spacing: 10) {
            Button {
                withAnimation(.spring(response: 0.4)) { store.toggle(item) }
            } label: {
                HStack(spacing: 10) {
                    Image(systemName: isCompleted ? "arrow.counterclockwise" : "checkmark")
                        .font(.body.weight(.semibold))
                    Text(isCompleted ? "Mark as Active" : "Mark as Completed")
                        .font(.body.weight(.semibold))
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 15)
                .background(isCompleted ? Color.orange : Color.green)
                .foregroundStyle(.white)
                .clipShape(RoundedRectangle(cornerRadius: 14))
                .shadow(color: (isCompleted ? Color.orange : Color.green).opacity(0.3), radius: 8, y: 4)
            }

            Button(role: .destructive) {
                withAnimation { store.delete(item) }
                dismiss()
            } label: {
                HStack(spacing: 10) {
                    Image(systemName: "trash")
                        .font(.body.weight(.semibold))
                    Text("Delete Task")
                        .font(.body.weight(.semibold))
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 15)
                .background(Color.red.opacity(0.08))
                .foregroundStyle(.red)
                .clipShape(RoundedRectangle(cornerRadius: 14))
            }
        }
    }

    // MARK: - Helpers

    private func badge(icon: String, text: String, color: Color) -> some View {
        HStack(spacing: 4) {
            Image(systemName: icon)
                .font(.caption2)
            Text(text)
                .font(.caption.weight(.medium))
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(color.opacity(0.12))
        .foregroundStyle(color)
        .clipShape(Capsule())
    }

    private func metaChip(icon: String, label: String, value: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .frame(width: 20)

            VStack(alignment: .leading, spacing: 1) {
                Text(label)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                Text(value)
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(.primary)
            }

            Spacer()
        }
        .padding(12)
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .shadow(color: .black.opacity(0.04), radius: 6, y: 2)
    }

    private func statCard(icon: String, label: String, value: String, color: Color) -> some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundStyle(color)

            Text(label)
                .font(.caption2)
                .foregroundStyle(.tertiary)

            Text(value)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.primary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 14)
        .background(Color(.systemBackground))
        .clipShape(RoundedRectangle(cornerRadius: 14))
        .shadow(color: .black.opacity(0.04), radius: 6, y: 2)
    }
}
