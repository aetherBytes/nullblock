import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import { Task } from '../types/tasks';

export const exportTaskToPDF = (task: Task): void => {
  const doc = new jsPDF();
  const pageWidth = doc.internal.pageSize.getWidth();
  let yPos = 20;

  doc.setFontSize(20);
  doc.setFont('helvetica', 'bold');
  doc.text('Task Report', pageWidth / 2, yPos, { align: 'center' });
  yPos += 15;

  doc.setFontSize(10);
  doc.setFont('helvetica', 'normal');
  doc.setTextColor(100);
  doc.text(`Generated: ${new Date().toLocaleString()}`, pageWidth / 2, yPos, { align: 'center' });
  yPos += 15;

  doc.setFontSize(16);
  doc.setFont('helvetica', 'bold');
  doc.setTextColor(0);
  doc.text(task.name, 15, yPos);
  yPos += 10;

  doc.setFontSize(10);
  doc.setFont('helvetica', 'normal');
  const splitDescription = doc.splitTextToSize(task.description, pageWidth - 30);
  doc.text(splitDescription, 15, yPos);
  yPos += splitDescription.length * 5 + 10;

  autoTable(doc, {
    startY: yPos,
    head: [['Field', 'Value']],
    body: [
      ['Task ID', task.id],
      ['Type', task.task_type],
      ['Category', task.category],
      ['Status', task.status.state],
      ['Priority', task.priority],
      ['Progress', `${task.progress}%`],
      ['Created', new Date(task.created_at).toLocaleString()],
      ...(task.started_at ? [['Started', new Date(task.started_at).toLocaleString()]] : []),
      ...(task.completed_at ? [['Completed', new Date(task.completed_at).toLocaleString()]] : []),
      ...(task.assigned_agent ? [['Assigned Agent', task.assigned_agent]] : []),
      ...(task.action_duration ? [['Duration', `${(task.action_duration / 1000).toFixed(2)}s`]] : []),
      ...(task.source_identifier ? [['Source', task.source_identifier]] : []),
    ],
    theme: 'grid',
    headStyles: { fillColor: [78, 205, 196] },
    styles: { fontSize: 9 },
  });

  yPos = (doc as any).lastAutoTable.finalY + 10;

  if (task.parameters?.preferred_model) {
    doc.setFontSize(12);
    doc.setFont('helvetica', 'bold');
    doc.text('Model Configuration', 15, yPos);
    yPos += 7;

    autoTable(doc, {
      startY: yPos,
      head: [['Parameter', 'Value']],
      body: [
        ['Preferred Model', task.parameters.preferred_model],
        ...(task.parameters.temperature !== undefined ? [['Temperature', String(task.parameters.temperature)]] : []),
        ...(task.parameters.max_tokens ? [['Max Tokens', String(task.parameters.max_tokens)]] : []),
        ...(task.parameters.timeout_ms ? [['Timeout', `${(task.parameters.timeout_ms / 1000).toFixed(0)}s`]] : []),
      ],
      theme: 'grid',
      headStyles: { fillColor: [230, 194, 0] },
      styles: { fontSize: 9 },
    });

    yPos = (doc as any).lastAutoTable.finalY + 10;
  }

  if (task.sub_tasks && task.sub_tasks.length > 0) {
    doc.setFontSize(12);
    doc.setFont('helvetica', 'bold');
    doc.text(`Sub-tasks (${task.sub_tasks.length})`, 15, yPos);
    yPos += 7;

    const subTaskRows = task.sub_tasks.map((st: any, idx: number) => [
      String(idx + 1),
      st.name || `Sub-task ${idx + 1}`,
      st.description || 'No description',
      st.assigned_agent_id || 'Auto'
    ]);

    autoTable(doc, {
      startY: yPos,
      head: [['#', 'Name', 'Description', 'Agent']],
      body: subTaskRows,
      theme: 'grid',
      headStyles: { fillColor: [78, 205, 196] },
      styles: { fontSize: 8 },
      columnStyles: {
        0: { cellWidth: 10 },
        1: { cellWidth: 40 },
        2: { cellWidth: 100 },
        3: { cellWidth: 30 }
      }
    });

    yPos = (doc as any).lastAutoTable.finalY + 10;
  }

  if (task.action_result) {
    if (yPos > 250) {
      doc.addPage();
      yPos = 20;
    }

    doc.setFontSize(12);
    doc.setFont('helvetica', 'bold');
    doc.text('Task Result', 15, yPos);
    yPos += 7;

    doc.setFontSize(9);
    doc.setFont('helvetica', 'normal');
    const resultLines = task.action_result.split('\n');
    resultLines.forEach((line: string) => {
      if (yPos > 280) {
        doc.addPage();
        yPos = 20;
      }
      const splitLine = doc.splitTextToSize(line || ' ', pageWidth - 30);
      doc.text(splitLine, 15, yPos);
      yPos += splitLine.length * 4;
    });
  }

  if (task.history && task.history.length > 0) {
    if (yPos > 250) {
      doc.addPage();
      yPos = 20;
    }

    doc.setFontSize(12);
    doc.setFont('helvetica', 'bold');
    doc.text('Conversation History', 15, yPos);
    yPos += 7;

    task.history.forEach((msg: any, idx: number) => {
      if (yPos > 270) {
        doc.addPage();
        yPos = 20;
      }

      doc.setFontSize(9);
      doc.setFont('helvetica', 'bold');
      doc.text(`${msg.role}: ${msg.timestamp || ''}`, 15, yPos);
      yPos += 5;

      doc.setFont('helvetica', 'normal');
      const textContent = msg.parts?.[0]?.text || JSON.stringify(msg.parts);
      const splitContent = doc.splitTextToSize(textContent, pageWidth - 30);
      doc.text(splitContent, 15, yPos);
      yPos += splitContent.length * 4 + 5;
    });
  }

  const fileName = `task-${task.name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}-${Date.now()}.pdf`;
  doc.save(fileName);
};
