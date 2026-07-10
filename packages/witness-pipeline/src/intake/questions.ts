export interface IntakeQuestion {
  header: string;
  question: string;
  options?: Array<{ label: string; description: string }>;
  multiple?: boolean;
}

export function buildReportIntakeQuestions(_opts: { subjectCount: number; relationship: boolean }): IntakeQuestion[] {
  return [
    {
      header: 'Report Type',
      question: 'What kind of report are we generating?',
      options: [
        { label: 'Individual', description: 'One person report' },
        { label: 'Synastry', description: 'Two or more people relationship mapping' },
      ],
    },
    {
      header: 'Gender',
      question: 'What gender should the report use for reader-facing language?',
      options: [
        { label: 'Female', description: 'Use female reader-facing language where relevant' },
        { label: 'Male', description: 'Use male reader-facing language where relevant' },
        { label: 'Nonbinary', description: 'Use neutral reader-facing language' },
        { label: 'Prefer not to say', description: 'Avoid gendered language' },
      ],
    },
    { header: 'Birthplace', question: 'What birthplace should we normalize into latitude, longitude, and timezone?' },
    {
      header: 'Confirm Location',
      question: 'Confirm the selected place, latitude, longitude, and timezone before report generation.',
      options: [
        { label: 'Use selected', description: 'Accept normalized location' },
        { label: 'Pick another', description: 'Choose from alternate geocoding results' },
        { label: 'Enter manually', description: 'Provide lat/long/timezone directly' },
      ],
    },
  ];
}

export function getLanguageQuestion(): IntakeQuestion {
  return {
    header: 'Language',
    question: 'In which language should the report and any generated assets be produced?',
    options: [
      { label: 'en', description: 'English (default)' },
      { label: 'hi', description: 'Hindi' },
      { label: 'es', description: 'Spanish' },
      // Add more as needed; free-text fallback allowed downstream
    ],
  };
}
