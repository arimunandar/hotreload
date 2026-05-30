import SwiftUI
import UIKit

// MARK: - Haptic Generator

private let impactFeedback = UIImpactFeedbackGenerator(style: .soft)
private let completionFeedback = UINotificationFeedbackGenerator()

// MARK: - Todo Row View

struct TodoRowView: View {
    let item: TodoItem
    let onToggle: () -> Void
    let onDelete: () -> Void

    @State private var isPressed = false
    @State private var isAppeared = false
    @State private var isToggling = false

    private var isCompleted: Bool { item.isCompleted }

    var body: some View {
        HStack(spacing: 14) {
            toggleButton
            contentBlock
            Spacer(minLength: 4)
            chevron
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 14)
        .background(cardBackground)
        .overlay(cardBorder)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .shadow(color: .black.opacity(0.06), radius: 8, y: 2)
        .shadow(color: .black.opacity(0.03), radius: 4, y: 1)
        .shadow(color: item.priority.color.opacity(isCompleted ? 0 : 0.08), radius: 6, y: 2)
        .scaleEffect(isPressed ? 0.975 : 1)
        .opacity(isAppeared ? 1 : 0)
        .offset(y: isAppeared ? 0 : 12)
        .onAppear(perform: animateAppear)
        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
            Button(role: .destructive, action: onDelete) {
                Label("Delete", systemImage: "trash")
            }
            .tint(.red)
        }
        .swipeActions(edge: .leading, allowsFullSwipe: true) {
            Button(action: triggerToggle) {
                Label(isCompleted ? "Undo" : "Done", systemImage: isCompleted ? "arrow.uturn.backward" : "checkmark")
            }
            .tint(isCompleted ? .orange : .green)
        }
        .onLongPressGesture(minimumDuration: .infinity, maximumDistance: .infinity, pressing: { pressing in
            withAnimation(.spring(response: 0.25, dampingFraction: 0.6)) {
                isPressed = pressing
            }
        }, perform: {})
    }

    // MARK: - Toggle Button

    private var toggleButton: some View {
        Button(action: triggerToggle) {
            ZStack {
                // Outer ring
                Circle()
                    .stroke(
                        isCompleted ? Color.green.opacity(0.5) : Color.gray.opacity(0.2),
                        lineWidth: 2
                    )
                    .frame(width: 24, height: 24)
                    .shadow(color: isCompleted ? .green.opacity(0.12) : .clear, radius: 4)

                // Fill ring
                Circle()
                    .trim(from: 0, to: isCompleted ? 1 : 0)
                    .stroke(
                        Color.green,
                        style: StrokeStyle(lineWidth: 2, lineCap: .round)
                    )
                    .frame(width: 24, height: 24)
                    .rotationEffect(.degrees(-90))

                // Inner fill
                if isCompleted {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 24, height: 24)
                        .shadow(color: .green.opacity(0.3), radius: 4, y: 1)
                        .transition(.scale(scale: 0.5).combined(with: .opacity))
                }

                // Checkmark
                Image(systemName: "checkmark")
                    .font(.system(size: 11, weight: .black))
                    .foregroundStyle(isCompleted ? .white : .clear)
                    .scaleEffect(isCompleted ? 1 : 0.3)
            }
        }
        .buttonStyle(.plain)
        .scaleEffect(isToggling ? 0.8 : 1)
        .animation(.spring(response: 0.3, dampingFraction: 0.5), value: isCompleted)
    }

    // MARK: - Content Block

    private var contentBlock: some View {
        VStack(alignment: .leading, spacing: 6) {
            titleText
            if !item.notes.isEmpty {
                notesPreview
            }
            metaRow
        }
    }

    // MARK: - Title

    private var titleText: some View {
        Text(item.title)
            .font(.system(size: 16, weight: .semibold, design: .rounded))
            .foregroundStyle(isCompleted ? .secondary : .primary)
            .strikethrough(isCompleted, pattern: .solid, color: .secondary.opacity(0.5))
            .lineLimit(1)
            .opacity(isCompleted ? 0.7 : 1)
    }

    // MARK: - Notes Preview

    private var notesPreview: some View {
        HStack(spacing: 5) {
            Image(systemName: "note.text")
                .font(.system(size: 9.5))
                .foregroundStyle(.quaternary)

            Text(item.notes)
                .font(.subheadline)
                .foregroundStyle(.tertiary)
                .lineLimit(1)
        }
        .opacity(isCompleted ? 0.5 : 1)
    }

    // MARK: - Meta Row

    private var metaRow: some View {
        HStack(spacing: 6) {
            // Category badge
            categoryBadge

            Text("\u{00B7}")
                .font(.caption2.weight(.black))
                .foregroundStyle(.quinary)

            // Priority indicator
            priorityIndicator

            Text("\u{00B7}")
                .font(.caption2.weight(.black))
                .foregroundStyle(.quinary)

            // Relative time
            HStack(spacing: 3) {
                Image(systemName: "clock")
                    .font(.system(size: 8))
                    .foregroundStyle(.quaternary)

                Text(item.createdAt, style: .relative)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
    }

    // MARK: - Category Badge

    private var categoryBadge: some View {
        HStack(spacing: 3) {
            Circle()
                .fill(categoryColor(for: item.category).opacity(0.8))
                .frame(width: 5, height: 5)

            Text(item.category)
                .font(.caption2.weight(.medium))
                .foregroundStyle(categoryColor(for: item.category).opacity(0.85))
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 2)
        .background(categoryColor(for: item.category).opacity(0.08))
        .clipShape(Capsule())
    }

    // MARK: - Priority Indicator

    private var priorityIndicator: some View {
        HStack(spacing: 3) {
            Image(systemName: item.priority.icon)
                .font(.system(size: 9))
                .foregroundStyle(item.priority.color.opacity(isCompleted ? 0.25 : 0.65))

            Text(item.priority.label)
                .font(.caption2.weight(.medium))
                .foregroundStyle(item.priority.color.opacity(isCompleted ? 0.25 : 0.55))
        }
    }

    // MARK: - Chevron

    private var chevron: some View {
        Image(systemName: "chevron.right")
            .font(.caption.weight(.semibold))
            .foregroundStyle(.quaternary)
    }

    // MARK: - Card Background

    private var cardBackground: some View {
        RoundedRectangle(cornerRadius: 16)
            .fill(Color(.systemBackground))
            .overlay(
                LinearGradient(
                    colors: [
                        .white.opacity(0.55),
                        .clear
                    ],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .clipShape(RoundedRectangle(cornerRadius: 16))
            )
    }

    private var borderColor: Color {
        if isCompleted {
            return Color.green.opacity(0.12)
        }
        return Color(.separator).opacity(0.12)
    }

    private var cardBorder: some View {
        RoundedRectangle(cornerRadius: 16)
            .stroke(borderColor, lineWidth: 1)
    }

    // MARK: - Category Colors

    private func categoryColor(for category: String) -> Color {
        switch category {
        case "Work":     return .indigo
        case "Personal": return .pink
        case "Shopping": return .orange
        case "Health":   return .mint
        case "Learning": return .teal
        default:         return .purple
        }
    }

    // MARK: - Animations

    private func animateAppear() {
        withAnimation(.spring(response: 0.5, dampingFraction: 0.75).delay(Double.random(in: 0.02...0.08))) {
            isAppeared = true
        }
    }

    private func triggerToggle() {
        impactFeedback.impactOccurred()

        withAnimation(.spring(response: 0.35, dampingFraction: 0.55)) {
            isToggling = true
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) {
            withAnimation(.spring(response: 0.4, dampingFraction: 0.6)) {
                isToggling = false
            }
            onToggle()
        }

        if !isCompleted {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                completionFeedback.notificationOccurred(.success)
            }
        }
    }
}

// MARK: - Preview
// Note: #Preview macro requires Xcode build system (not available in standalone swiftc)
// For Xcode previews, build the project normally.
